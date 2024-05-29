use actix::{AsyncContext, Handler};

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadTaskState as State},
    messages::CancelTaskMessage,
};

impl Handler<CancelTaskMessage> for Task {
    type Result = ();
    fn handle(&mut self, _msg: CancelTaskMessage, ctx: &mut Self::Context) -> Self::Result {
        if let Some(handle) = self.handle.take() {
            ctx.cancel_future(handle);
        }
        self.sender.send_replace(State::Canceled);
    }
}
