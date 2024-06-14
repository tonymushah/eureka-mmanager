use actix::prelude::*;
use mangadex_api::MangaDexClient;

use std::mem;

use crate::download::state::DownloadManagerState;

#[derive(Debug, Clone)]
pub struct UpdateClientMessage(pub MangaDexClient);

impl Message for UpdateClientMessage {
    type Result = MangaDexClient;
}

impl Handler<UpdateClientMessage> for DownloadManagerState {
    type Result = <UpdateClientMessage as Message>::Result;
    fn handle(&mut self, msg: UpdateClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        mem::replace(&mut self.client, msg.0)
    }
}
