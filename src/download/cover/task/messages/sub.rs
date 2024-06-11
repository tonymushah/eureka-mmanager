use actix::prelude::*;

use crate::download::{
    cover::task::{CoverDownloadTask as Task, CoverDownloadTaskState as State},
    messages::SubcribeMessage,
    traits::Subscribe,
};

impl Handler<SubcribeMessage<State>> for Task {
    type Result = <SubcribeMessage<State> as Message>::Result;
    fn handle(&mut self, _msg: SubcribeMessage<State>, _ctx: &mut Self::Context) -> Self::Result {
        self.subscribe()
    }
}

impl Subscribe for Task {
    fn subscribe(&mut self) -> crate::ManagerCoreResult<tokio::sync::watch::Receiver<Self::State>> {
        if !self.have_been_read {
            self.have_been_read = true;
        }
        Ok(self.sender.subscribe())
    }
}
