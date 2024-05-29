use actix::prelude::*;
use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::RelationshipType;

use crate::{
    data_push::cover::required_cover_references,
    download::{
        cover::task::{CoverDownloadTask as Task, CoverDownloadingState as State},
        messages::{state::GetManagerStateMessage, StartDownload, TaskStateMessage},
        state::{
            messages::get::{
                client::GetClientMessage, dir_options::GetDirsOptionsMessage,
                history::GetHistoryMessage,
            },
            DownloadTaskState, TaskState,
        },
    },
    files_dirs::messages::push::PushDataMessage,
    history::{
        service::messages::{insert::InsertMessage, remove::RemoveMessage},
        HistoryEntry,
    },
    ManagerCoreResult,
};

impl Handler<StartDownload> for Task {
    type Result = ();
    fn handle(&mut self, _msg: StartDownload, ctx: &mut Self::Context) -> Self::Result {
        if self.handle(TaskStateMessage, ctx) != TaskState::Loading {
            self.sender
                .send_replace(DownloadTaskState::Loading(State::Preloading));
            let manager = self.manager.clone();

            let sender = self.sender.clone();
            let id = self.id;

            let entry = HistoryEntry::new(id, RelationshipType::CoverArt);
            if let Some(t) = self.handle.replace(
                ctx.spawn(
                    async move {
                        let manager_state = manager.send(GetManagerStateMessage).await?;
                        let client = manager_state.send(GetClientMessage).await?;
                        let dir_options = manager_state.send(GetDirsOptionsMessage).await?;
                        let history = manager_state.send(GetHistoryMessage).await?;
                        sender.send_replace(DownloadTaskState::Loading(State::FetchingData));
                        history.send(InsertMessage::new(entry).commit()).await??;
                        let res = client
                            .cover()
                            .cover_id(id)
                            .get()
                            .includes(required_cover_references())
                            .send()
                            .await?;
                        dir_options
                            .send(PushDataMessage::new(res.data.clone()).verify(true))
                            .await??;
                        sender.send_replace(DownloadTaskState::Loading(State::FetchingImage));
                        let (_, image) = client
                            .download()
                            .cover()
                            .build()?
                            .via_cover_api_object(res.data.clone())
                            .await;
                        dir_options
                            .send(PushDataMessage::new((res.data.clone(), image?)).verify(true))
                            .await??;
                        history.send(RemoveMessage::new(entry).commit()).await??;
                        Ok(res.data)
                    }
                    .into_actor(self)
                    .map(|res: ManagerCoreResult<CoverObject>, this, _| match res {
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
