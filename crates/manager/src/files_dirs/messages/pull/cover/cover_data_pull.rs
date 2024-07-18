use actix::prelude::*;
use mangadex_api_schema_rust::v5::CoverObject;
use uuid::Uuid;

use crate::{data_pulls::Pull, DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Hash, Default)]
pub struct CoverDataPullMessage(pub Uuid);

impl From<Uuid> for CoverDataPullMessage {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<CoverDataPullMessage> for Uuid {
    fn from(value: CoverDataPullMessage) -> Self {
        value.0
    }
}

impl Message for CoverDataPullMessage {
    type Result = ManagerCoreResult<CoverObject>;
}

impl Handler<CoverDataPullMessage> for DirsOptions {
    type Result = <CoverDataPullMessage as Message>::Result;
    fn handle(&mut self, msg: CoverDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        (**self)
            .pull(msg.into())
            .map_err(|e: api_core::Error| e.into())
    }
}
