use actix::prelude::*;
use mangadex_api_schema_rust::v5::CoverObject as Object;

use crate::download::{
    cover::task::{CoverDownloadTask as Task, CoverDownloadingState as State},
    messages::WaitForFinishedMessage,
    state::{make_wait_for_finish_couple, WaitForFinished},
    traits::task::{CanBeWaited, Subscribe},
};

pub type WaitForFinishedCoverMessage = WaitForFinishedMessage<Object, State>;

impl CanBeWaited for Task {
    type Loading = State;
    type Ok = Object;
    fn wait(&mut self) -> WaitForFinished<Self::Ok, Self::Loading> {
        let (rc, wait) = make_wait_for_finish_couple::<Self::Ok, Self::Loading>();
        self.subscribe(rc.into());
        wait
    }
}

impl Handler<WaitForFinishedCoverMessage> for Task {
    type Result = <WaitForFinishedCoverMessage as Message>::Result;
    fn handle(
        &mut self,
        _msg: WaitForFinishedCoverMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.wait()
    }
}
