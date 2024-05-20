pub mod new_task;

use actix::{Handler, Message};
pub use new_task::MangaDownloadMessage;

use crate::download::messages::state::GetManagerStateMessage;

use super::MangaDownloadManager;

impl Handler<GetManagerStateMessage> for MangaDownloadManager {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state.clone()
    }
}
