pub mod new_task;
pub mod tasks;

use actix::{Handler, Message};

use crate::download::messages::{state::GetManagerStateMessage, SubcribeToManagerMessage};

use super::CoverDownloadManager as Manager;

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
