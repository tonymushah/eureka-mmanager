use actix::prelude::*;
use bytes::Buf;
use futures_util::FutureExt;
use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::RelationshipType;

use crate::{
    data_push::cover::required_cover_references,
    download::{
        cover::task::{CoverDownloadTask as Task, CoverDownloadingState as State},
        messages::StartDownload,
        state::{messages::get::GetManagerStateData, DownloadTaskState, TaskState},
        traits::task::{Download, State as TaskStateTrait},
    },
    history::{
        history_w_file::traits::{AsyncAutoCommitRollbackInsert, AsyncAutoCommitRollbackRemove},
        HistoryEntry,
    },
    prelude::PushActorAddr,
    ManagerCoreResult,
};

impl Task {
    fn preloading(&self) {
        self.send_to_subscrbers()(DownloadTaskState::Loading(State::Preloading));
    }
}

impl Download for Task {
    fn download(&mut self, ctx: &mut Self::Context) {
        if self.state() != TaskState::Loading {
            self.preloading();
            let manager = self.manager.clone();

            let send_to_subscribers = self.send_to_subscrbers();
            let send_to_subscribers2 = send_to_subscribers.clone();
            let id = self.id;

            let entry = HistoryEntry::new(id, RelationshipType::CoverArt);
            if let Some(t) = self.handle.replace(
                ctx.spawn(
                    async move {
                        let client = manager.get_client().await?;
                        let mut history = manager.get_history().await?;
                        send_to_subscribers(DownloadTaskState::Loading(State::FetchingData));
                        history.insert_and_commit(entry).await?;
                        let res = client
                            .cover()
                            .cover_id(id)
                            .get()
                            .includes(required_cover_references())
                            .send()
                            .await?;
                        manager.verify_and_push(res.data.clone()).await?;
                        send_to_subscribers(DownloadTaskState::Loading(State::FetchingImage));
                        let (_, image) = client
                            .download()
                            .cover()
                            .build()?
                            .via_cover_api_object(res.data.clone())
                            .await;
                        manager.push((res.data.clone(), image?.reader())).await?;
                        history.remove_and_commit(entry).await?;
                        Ok(res.data)
                    }
                    .map(move |res: ManagerCoreResult<CoverObject>| match res {
                        Ok(data) => {
                            send_to_subscribers2(DownloadTaskState::Done(data));
                        }
                        Err(err) => {
                            send_to_subscribers2(DownloadTaskState::Error(err.into()));
                        }
                    })
                    .into_actor(self),
                ),
            ) {
                ctx.cancel_future(t);
            }
        }
    }
}

impl Handler<StartDownload> for Task {
    type Result = ();
    fn handle(&mut self, _msg: StartDownload, ctx: &mut Self::Context) -> Self::Result {
        self.download(ctx);
    }
}
