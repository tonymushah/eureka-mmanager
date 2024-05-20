use actix::prelude::*;

use crate::{download::state::DownloadManagerState, history::service::HistoryActorService};

#[derive(Debug, Clone)]
pub struct UpdateHistoryMessage(pub Addr<HistoryActorService>);

impl Message for UpdateHistoryMessage {
    type Result = ();
}

impl Handler<UpdateHistoryMessage> for DownloadManagerState {
    type Result = <UpdateHistoryMessage as Message>::Result;
    fn handle(&mut self, msg: UpdateHistoryMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.history = msg.0;
    }
}
