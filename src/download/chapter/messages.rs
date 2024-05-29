pub mod new_task;
pub use self::new_task::ChapterDownloadMessage;

use actix::{Handler, Message};

use crate::download::messages::{state::GetManagerStateMessage, SubcribeToManagerMessage};

use super::ChapterDownloadManager as Manager;

impl Handler<GetManagerStateMessage> for Manager {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state.clone()
    }
}

impl Handler<SubcribeToManagerMessage> for Manager {
    type Result = <SubcribeToManagerMessage as Message>::Result;
    fn handle(&mut self, _msg: SubcribeToManagerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.notify.clone()
    }
}
