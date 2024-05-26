use actix::{AsyncContext, Handler};

use crate::download::{
    cover::task::{CoverDownloadTask as Task, CoverDownloadTaskState as State},
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
