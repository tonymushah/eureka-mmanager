use std::ops::Deref;

use actix::prelude::*;

use crate::download::{
    manga::task::{MangaDownloadTask, MangaDownloadTaskState},
    messages::TaskStateMessage,
    state::TaskState,
    traits::task::State,
};

impl State for MangaDownloadTask {
    type State = MangaDownloadTaskState;
    fn inner_state(&self) -> Self::State {
        self.sender.borrow().deref().clone()
    }
}

impl Handler<TaskStateMessage> for MangaDownloadTask {
    type Result = TaskState;
    fn handle(&mut self, _msg: TaskStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state()
    }
}
