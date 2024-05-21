use actix::prelude::*;

use crate::download::{manga::MangaDownloadManager, messages::GetTasksListMessage};

impl Handler<GetTasksListMessage> for MangaDownloadManager {
    type Result = <GetTasksListMessage as Message>::Result;
    fn handle(&mut self, _msg: GetTasksListMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks.keys().copied().collect()
    }
}
