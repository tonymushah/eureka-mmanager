use actix::prelude::*;

use crate::download::{cover::CoverDownloadManager as Manager, messages::GetTasksListMessage};

impl Handler<GetTasksListMessage> for Manager {
    type Result = <GetTasksListMessage as Message>::Result;
    fn handle(&mut self, _msg: GetTasksListMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks.keys().copied().collect()
    }
}
