use actix::prelude::*;
use uuid::Uuid;

use crate::{data_pulls::chapter::ids::ChapterIdsListDataPull, DirsOptions};

#[derive(Debug, Clone, Hash, Default)]
pub struct ChapterIdsListDataPullMessage(pub Vec<Uuid>);

impl From<Vec<Uuid>> for ChapterIdsListDataPullMessage {
    fn from(value: Vec<Uuid>) -> Self {
        Self(value)
    }
}

impl From<ChapterIdsListDataPullMessage> for Vec<Uuid> {
    fn from(value: ChapterIdsListDataPullMessage) -> Self {
        value.0
    }
}

impl Message for ChapterIdsListDataPullMessage {
    type Result = ChapterIdsListDataPull;
}

impl Handler<ChapterIdsListDataPullMessage> for DirsOptions {
    type Result = ChapterIdsListDataPull;
    fn handle(
        &mut self,
        msg: ChapterIdsListDataPullMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.pull_chapter_ids(msg.into())
    }
}
