use actix::{Handler, Message};

use crate::download::messages::{
    state::GetManagerStateMessage, GetTasksListMessage, SubcribeToManagerMessage,
};

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

impl Handler<GetTasksListMessage> for MangaDownloadManager {
    type Result = <GetTasksListMessage as Message>::Result;
    fn handle(&mut self, _msg: GetTasksListMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks.keys().copied().collect()
    }
}
