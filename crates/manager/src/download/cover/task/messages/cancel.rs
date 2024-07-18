use actix::{AsyncContext, Handler};

use crate::download::{
    cover::task::{CoverDownloadTask as Task, CoverDownloadTaskState as State},
    messages::CancelTaskMessage,
    traits::task::Cancelable,
};

impl Handler<CancelTaskMessage> for Task {
    type Result = ();
    fn handle(&mut self, _msg: CancelTaskMessage, ctx: &mut Self::Context) -> Self::Result {
        self.cancel(ctx);
    }
}

impl Cancelable for Task {
    fn cancel(&mut self, ctx: &mut Self::Context) {
        if let Some(handle) = self.handle.take() {
            ctx.cancel_future(handle);
        }
        self.sender.send_replace(State::Canceled);
    }
}
