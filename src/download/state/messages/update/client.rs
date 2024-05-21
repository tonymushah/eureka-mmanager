use actix::prelude::*;
use mangadex_api::MangaDexClient;

use crate::download::state::DownloadManagerState;

#[derive(Debug, Clone)]
pub struct UpdateClientMessage(pub MangaDexClient);

impl Message for UpdateClientMessage {
    type Result = ();
}

impl Handler<UpdateClientMessage> for DownloadManagerState {
    type Result = <UpdateClientMessage as Message>::Result;
    fn handle(&mut self, msg: UpdateClientMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.client = msg.0;
    }
}
