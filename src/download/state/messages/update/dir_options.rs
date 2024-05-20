use actix::prelude::*;

use crate::{download::state::DownloadManagerState, DirsOptions};

#[derive(Debug, Clone)]
pub struct UpdateDirOptionsMessage(pub Addr<DirsOptions>);

impl Message for UpdateDirOptionsMessage {
    type Result = ();
}

impl Handler<UpdateDirOptionsMessage> for DownloadManagerState {
    type Result = <UpdateDirOptionsMessage as Message>::Result;
    fn handle(&mut self, msg: UpdateDirOptionsMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.dir_option = msg.0;
    }
}
