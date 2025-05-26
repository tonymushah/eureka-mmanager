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

impl Download for Task {
    fn download(&mut self, ctx: &mut Self::Context) {
        if self.state() != TaskState::Loading {
            self.sender
                .send_replace(DownloadTaskState::Loading(State::Preloading));
            let manager = self.manager.clone();

            let sender = self.sender.clone();
            let id = self.id;

            let entry = HistoryEntry::new(id, RelationshipType::CoverArt);
            let sender2 = sender.clone();
            if let Some(t) = self.handle.replace(
                ctx.spawn(
                    async move {
                        let client = manager.get_client().await?;
                        let mut history = manager.get_history().await?;
                        sender.send_replace(DownloadTaskState::Loading(State::FetchingData));
                        history.insert_and_commit(entry).await?;
                        let res = client
                            .cover()
                            .cover_id(id)
                            .get()
                            .includes(required_cover_references())
                            .send()
                            .await?;
                        manager.verify_and_push(res.data.clone()).await?;
                        sender.send_replace(DownloadTaskState::Loading(State::FetchingImage));
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
                            let _ = sender2.send(DownloadTaskState::Done(data));
                        }
                        Err(err) => {
                            let _ = sender2.send_replace(DownloadTaskState::Error(err.into()));
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
