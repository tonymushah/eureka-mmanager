use actix::prelude::*;
use futures_util::FutureExt;
use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::RelationshipType;

use crate::{
    data_push::manga::MangaRequiredRelationship,
    download::{
        manga::task::{MangaDonwloadingState, MangaDownloadTask},
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

impl Download for MangaDownloadTask {
    fn download(&mut self, ctx: &mut Self::Context) {
        if self.state() != TaskState::Loading {
            self.sender.send_replace(DownloadTaskState::Loading(
                MangaDonwloadingState::Preloading,
            ));
            let manager = self.manager.clone();

            let sender = self.sender.clone();
            let sender2 = sender.clone();
            let id = self.id;

            let entry = HistoryEntry::new(id, RelationshipType::Manga);
            if let Some(t) = self.handle.replace(
                ctx.spawn(
                    async move {
                        let client = manager.get_client().await?;
                        let mut history = manager.get_history().await?;
                        sender.send_replace(DownloadTaskState::Loading(
                            MangaDonwloadingState::FetchingData,
                        ));
                        history.insert_and_commit(entry).await?;
                        let res = client
                            .manga()
                            .id(id)
                            .get()
                            .includes(MangaRequiredRelationship::get_includes())
                            .send()
                            .await?;
                        manager.verify_and_push(res.data.clone()).await?;
                        history.remove_and_commit(entry).await?;
                        Ok(res.data)
                    }
                    .map(move |res: ManagerCoreResult<MangaObject>| match res {
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

impl Handler<StartDownload> for MangaDownloadTask {
    type Result = ();
    fn handle(&mut self, _msg: StartDownload, ctx: &mut Self::Context) -> Self::Result {
        self.download(ctx)
    }
}
