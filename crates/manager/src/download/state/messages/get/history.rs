use actix::prelude::*;

use crate::{download::state::DownloadManagerState, history::service::HistoryActorService};

#[derive(Debug, Clone, Copy, Default)]
pub struct GetHistoryMessage;

impl Message for GetHistoryMessage {
    type Result = Addr<HistoryActorService>;
}

impl Handler<GetHistoryMessage> for DownloadManagerState {
    type Result = <GetHistoryMessage as Message>::Result;
    fn handle(&mut self, _msg: GetHistoryMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.history.clone()
    }
}
