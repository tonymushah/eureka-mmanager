use actix::prelude::*;

use crate::download::{
    chapter::task::{ChapterDownloadTask as Task, ChapterDownloadTaskState as State},
    messages::SubcribeMessage,
    traits::task::Subscribe,
};

impl Handler<SubcribeMessage<State>> for Task {
    type Result = <SubcribeMessage<State> as Message>::Result;
    fn handle(&mut self, _msg: SubcribeMessage<State>, _ctx: &mut Self::Context) -> Self::Result {
        self.subscribe()
    }
}

impl Subscribe for Task {
    fn subscribe(&mut self) -> crate::ManagerCoreResult<tokio::sync::watch::Receiver<Self::State>> {
        Ok(self.sender.subscribe())
    }
}
