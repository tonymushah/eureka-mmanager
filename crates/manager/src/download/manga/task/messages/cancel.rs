use actix::{AsyncContext, Handler};

use crate::download::{
    manga::task::{MangaDownloadTask as Task, MangaDownloadTaskState as State},
    messages::CancelTaskMessage,
    traits::task::Cancelable,
};

impl Handler<CancelTaskMessage> for Task {
    type Result = ();
    fn handle(&mut self, _msg: CancelTaskMessage, ctx: &mut Self::Context) -> Self::Result {
        self.cancel(ctx)
    }
}

impl Cancelable for Task {
    fn cancel(&mut self, ctx: &mut Self::Context) {
        if let Some(handle) = self.handle.take() {
            ctx.cancel_future(handle);
        }
        self.send_to_subscrbers()(State::Canceled);
    }
}
