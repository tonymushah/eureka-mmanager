pub mod new_task;
pub mod tasks;

use actix::{Handler, Message};
pub use new_task::MangaDownloadMessage;

use crate::download::messages::{state::GetManagerStateMessage, SubcribeToManagerMessage};

use super::MangaDownloadManager;

impl Handler<GetManagerStateMessage> for MangaDownloadManager {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state.clone()
    }
}

impl Handler<SubcribeToManagerMessage> for MangaDownloadManager {
    type Result = <SubcribeToManagerMessage as Message>::Result;
    fn handle(&mut self, _msg: SubcribeToManagerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.notify.clone()
    }
}
