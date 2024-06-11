use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::RelationshipType;

use crate::{
    data_push::manga::MangaRequiredRelationship,
    download::{
        manga::task::{MangaDonwloadingState, MangaDownloadTask},
        messages::{state::GetManagerStateMessage, StartDownload, TaskStateMessage},
        state::{
            messages::get::{
                client::GetClientMessage, dir_options::GetDirsOptionsMessage,
                history::GetHistoryMessage,
            },
            DownloadTaskState, TaskState,
        },
        traits::Download,
    },
    files_dirs::messages::push::PushDataMessage,
    history::{
        service::messages::{insert::InsertMessage, remove::RemoveMessage},
        HistoryEntry,
    },
    ManagerCoreResult,
};

impl Download for MangaDownloadTask {
    fn download(&mut self, ctx: &mut Self::Context) {
        if self.handle(TaskStateMessage, ctx) != TaskState::Loading {
            self.sender.send_replace(DownloadTaskState::Loading(
                MangaDonwloadingState::Preloading,
            ));
            let manager = self.manager.clone();

            let sender = self.sender.clone();
            let id = self.id;

            let entry = HistoryEntry::new(id, RelationshipType::Manga);
            if let Some(t) = self.handle.replace(
                ctx.spawn(
                    async move {
                        let manager_state = manager.send(GetManagerStateMessage).await?;
                        let client = manager_state.send(GetClientMessage).await?;
                        let dir_options = manager_state.send(GetDirsOptionsMessage).await?;
                        let history = manager_state.send(GetHistoryMessage).await?;
                        sender.send_replace(DownloadTaskState::Loading(
                            MangaDonwloadingState::FetchingData,
                        ));
                        history.send(InsertMessage::new(entry).commit()).await??;
                        let res = client
                            .manga()
                            .id(id)
                            .get()
                            .includes(MangaRequiredRelationship::get_includes())
                            .send()
                            .await?;
                        dir_options
                            .send(PushDataMessage::new(res.data.clone()).verify(true))
                            .await??;
                        history.send(RemoveMessage::new(entry).commit()).await??;
                        Ok(res.data)
                    }
                    .into_actor(self)
                    .map(|res: ManagerCoreResult<MangaObject>, this, _| match res {
                        Ok(data) => {
                            let _ = this.sender.send(DownloadTaskState::Done(data));
                        }
                        Err(err) => {
                            let _ = this
                                .sender
                                .send_replace(DownloadTaskState::Error(err.into()));
                        }
                    }),
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
