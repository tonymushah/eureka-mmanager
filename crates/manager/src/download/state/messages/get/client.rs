use actix::prelude::*;
use mangadex_api::MangaDexClient;

use crate::download::state::DownloadManagerState;

#[derive(Debug, Clone, Copy, Default)]
pub struct GetClientMessage;

impl Message for GetClientMessage {
    type Result = MangaDexClient;
}

impl Handler<GetClientMessage> for DownloadManagerState {
    type Result = <GetClientMessage as Message>::Result;
    fn handle(&mut self, _msg: GetClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.client.clone()
    }
}
