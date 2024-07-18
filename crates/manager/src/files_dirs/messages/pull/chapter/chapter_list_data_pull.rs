use actix::prelude::*;

use crate::{data_pulls::chapter::list::ChapterListDataPull, DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct ChapterListDataPullMessage;

impl Message for ChapterListDataPullMessage {
    type Result = ManagerCoreResult<ChapterListDataPull>;
}

impl Handler<ChapterListDataPullMessage> for DirsOptions {
    type Result = ManagerCoreResult<ChapterListDataPull>;
    fn handle(
        &mut self,
        _msg: ChapterListDataPullMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.pull_all_chapter().map_err(|e| e.into())
    }
}
