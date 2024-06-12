use actix::{Handler, Message};

use crate::download::{
    messages::{state::GetManagerStateMessage, GetTasksListMessage, SubcribeToManagerMessage},
    traits::managers::TaskManager,
};

use super::ChapterDownloadManager as Manager;

impl Handler<GetManagerStateMessage> for Manager {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state()
    }
}

impl Handler<SubcribeToManagerMessage> for Manager {
    type Result = <SubcribeToManagerMessage as Message>::Result;
    fn handle(&mut self, _msg: SubcribeToManagerMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.notify()
    }
}

impl Handler<GetTasksListMessage> for Manager {
    type Result = <GetTasksListMessage as Message>::Result;
    fn handle(&mut self, _msg: GetTasksListMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks_id()
    }
}
