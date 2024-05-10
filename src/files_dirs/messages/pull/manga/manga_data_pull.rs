use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;
use uuid::Uuid;

use crate::{data_pulls::Pull, DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Hash, Default)]
pub struct MangaDataPullMessage(pub Uuid);

impl From<Uuid> for MangaDataPullMessage {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<MangaDataPullMessage> for Uuid {
    fn from(value: MangaDataPullMessage) -> Self {
        value.0
    }
}

impl Message for MangaDataPullMessage {
    type Result = ManagerCoreResult<MangaObject>;
}

impl Handler<MangaDataPullMessage> for DirsOptions {
    type Result = <MangaDataPullMessage as Message>::Result;

    fn handle(&mut self, msg: MangaDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.data_pull().pull(msg.into())
    }
}
