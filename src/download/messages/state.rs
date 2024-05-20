use actix::prelude::*;

use crate::download::{state::DownloadManagerState, DownloadManager};

#[derive(Debug, Clone, Copy, Default)]
pub struct GetManagerStateMessage;

impl Message for GetManagerStateMessage {
    type Result = Addr<DownloadManagerState>;
}

impl Handler<GetManagerStateMessage> for DownloadManagerState {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.address()
    }
}

impl Handler<GetManagerStateMessage> for DownloadManager {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state.clone()
    }
}
