use std::ops::Deref;

use actix::prelude::*;

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadTaskState},
    messages::TaskStateMessage,
    state::TaskState,
    traits::State,
};

impl Handler<TaskStateMessage> for Task {
    type Result = TaskState;
    fn handle(&mut self, _msg: TaskStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state()
    }
}

impl State for Task {
    type State = ChapterDownloadTaskState;
    fn inner_state(&self) -> Self::State {
        self.sender.borrow().deref().clone()
    }
}
