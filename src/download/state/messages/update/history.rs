use actix::prelude::*;

use std::mem;

use crate::{download::state::DownloadManagerState, history::service::HistoryActorService};

#[derive(Debug, Clone)]
pub struct UpdateHistoryMessage(pub Addr<HistoryActorService>);

impl Message for UpdateHistoryMessage {
    type Result = Addr<HistoryActorService>;
}

impl Handler<UpdateHistoryMessage> for DownloadManagerState {
    type Result = <UpdateHistoryMessage as Message>::Result;
    fn handle(&mut self, msg: UpdateHistoryMessage, _ctx: &mut Self::Context) -> Self::Result {
        mem::replace(&mut self.history, msg.0)
    }
}
