use actix::{AsyncContext, Handler};

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadTaskState as State},
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
        *self.state.write() = State::Canceled;
        self.sync_state_subscribers();
    }
}
