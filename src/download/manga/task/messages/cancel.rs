use actix::{AsyncContext, Handler};

use crate::download::{
    manga::task::{MangaDownloadTask, MangaDownloadTaskState},
    messages::CancelTaskMessage,
};

impl Handler<CancelTaskMessage> for MangaDownloadTask {
    type Result = ();
    fn handle(&mut self, _msg: CancelTaskMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Some(handle) = self.handle.take() {
            ctx.cancel_future(handle);
        }
        self.sender.send_replace(MangaDownloadTaskState::Canceled);
    }
}
