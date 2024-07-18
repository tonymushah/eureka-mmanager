use actix::prelude::*;

use crate::{data_pulls::cover::list::CoverListDataPull, DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct CoverListDataPullMessage;

impl Message for CoverListDataPullMessage {
    type Result = ManagerCoreResult<CoverListDataPull>;
}

impl Handler<CoverListDataPullMessage> for DirsOptions {
    type Result = ManagerCoreResult<CoverListDataPull>;
    fn handle(&mut self, _msg: CoverListDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.pull_all_covers().map_err(|e| e.into())
    }
}
