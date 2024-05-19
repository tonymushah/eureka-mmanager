use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::RelationshipType;

use crate::{
    data_push::manga::MangaRequiredRelationship,
    download::{
        manga::task::{MangaDonwloadingState, MangaDownloadTask},
        messages::{StartDownload, TaskStateMessage},
        state::{DownloadTaskState, TaskState},
    },
    files_dirs::messages::push::PushDataMessage,
    history::{
        service::messages::{insert::InsertMessage, remove::RemoveMessage},
        HistoryEntry,
    },
    ManagerCoreResult,
};

impl Handler<StartDownload> for MangaDownloadTask {
    type Result = ();
    fn handle(&mut self, _msg: StartDownload, ctx: &mut Self::Context) -> Self::Result {
        if self.handle(TaskStateMessage, ctx) != TaskState::Loading {
            let id = self.id;
            let client = self.client.clone();
            let dir_options = self.dir_option.clone();
            let sender = self.sender.clone();
            let history = self.history.clone();
            let entry = HistoryEntry::new(id, RelationshipType::Manga);
            self.handle.replace(
                ctx.spawn(
                    async move {
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
                            let _ = this.sender.send_replace(DownloadTaskState::Error(err));
                        }
                    }),
                ),
            );
        }
    }
}
