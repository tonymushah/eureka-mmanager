use actix::prelude::*;
use mangadex_api_schema_rust::v5::ChapterObject;
use uuid::Uuid;

use crate::{data_pulls::Pull, DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Hash, Default)]
pub struct ChapterDataPullMessage(pub Uuid);

impl From<Uuid> for ChapterDataPullMessage {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<ChapterDataPullMessage> for Uuid {
    fn from(value: ChapterDataPullMessage) -> Self {
        value.0
    }
}

impl Message for ChapterDataPullMessage {
    type Result = ManagerCoreResult<ChapterObject>;
}

impl Handler<ChapterDataPullMessage> for DirsOptions {
    type Result = <ChapterDataPullMessage as Message>::Result;
    // TODO add cbor support
    fn handle(&mut self, msg: ChapterDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.pull(msg.into())
    }
}
