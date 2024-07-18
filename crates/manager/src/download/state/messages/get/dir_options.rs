use actix::prelude::*;

use crate::{download::state::DownloadManagerState, DirsOptions};

#[derive(Debug, Clone, Copy, Default)]
pub struct GetDirsOptionsMessage;

impl Message for GetDirsOptionsMessage {
    type Result = Addr<DirsOptions>;
}

impl Handler<GetDirsOptionsMessage> for DownloadManagerState {
    type Result = <GetDirsOptionsMessage as Message>::Result;
    fn handle(&mut self, _msg: GetDirsOptionsMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.dir_option.clone()
    }
}
