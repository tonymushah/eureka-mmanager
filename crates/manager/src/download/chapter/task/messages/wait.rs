use actix::prelude::*;
use mangadex_api_schema_rust::v5::ChapterObject as Object;

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadingState as State},
    messages::WaitForFinishedMessage,
    state::{make_wait_for_finish_couple, WaitForFinished},
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
        let (recipient, fut) = make_wait_for_finish_couple::<Self::Ok, Self::Loading>();
        self.subscribers.push_recipient(recipient.into());
        fut
    }
}
