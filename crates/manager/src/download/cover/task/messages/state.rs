use std::ops::Deref;

use actix::prelude::*;

use crate::download::{
    cover::task::{CoverDownloadTask as Task, CoverDownloadTaskState},
    messages::TaskStateMessage,
    state::TaskState,
    traits::task::State,
};

impl Handler<TaskStateMessage> for Task {
    type Result = TaskState;
    fn handle(&mut self, _msg: TaskStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state()
    }
}

impl State for Task {
    type State = CoverDownloadTaskState;
    fn inner_state(&self) -> Self::State {
        self.state.read().deref().clone()
    }
}
