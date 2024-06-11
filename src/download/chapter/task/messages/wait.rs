use actix::prelude::*;
use mangadex_api_schema_rust::v5::ChapterObject as Object;

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadingState as State},
    messages::WaitForFinishedMessage,
    state::WaitForFinished,
    traits::task::CanBeWaited,
};

pub type WaitForFinishedChapterMessage = WaitForFinishedMessage<Object, State>;

impl Handler<WaitForFinishedChapterMessage> for Task {
    type Result = <WaitForFinishedChapterMessage as Message>::Result;
    fn handle(
        &mut self,
        _msg: WaitForFinishedMessage<Object, State>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.wait()
    }
}

impl CanBeWaited for Task {
    type Ok = Object;
    type Loading = State;
    fn wait(&mut self) -> WaitForFinished<Self::Ok, Self::Loading> {
        if !self.have_been_read {
            self.have_been_read = true;
        }
        WaitForFinished::new(self.sender.subscribe())
    }
}
