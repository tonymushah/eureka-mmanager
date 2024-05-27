use std::ops::Deref;

use actix::prelude::*;

use crate::download::{
    chapter::task::ChapterDownloadTask as Task, messages::TaskStateMessage, state::TaskState,
};

impl Handler<TaskStateMessage> for Task {
    type Result = TaskState;
    fn handle(&mut self, _msg: TaskStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.sender.borrow().deref().into()
    }
}
