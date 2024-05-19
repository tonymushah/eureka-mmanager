use std::ops::Deref;

use actix::prelude::*;

use crate::download::{
    manga::task::MangaDownloadTask, messages::TaskStateMessage, state::TaskState,
};

impl Handler<TaskStateMessage> for MangaDownloadTask {
    type Result = TaskState;
    fn handle(&mut self, _msg: TaskStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.sender.borrow().deref().into()
    }
}
