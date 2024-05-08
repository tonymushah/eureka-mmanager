use actix::prelude::*;
use uuid::Uuid;

use crate::{data_pulls::cover::ids::CoverIdsListDataPull, DirsOptions};

#[derive(Debug, Clone, Hash, Default)]
pub struct CoverIdsListDataPullMessage(pub Vec<Uuid>);

impl From<Vec<Uuid>> for CoverIdsListDataPullMessage {
    fn from(value: Vec<Uuid>) -> Self {
        Self(value)
    }
}

impl From<CoverIdsListDataPullMessage> for Vec<Uuid> {
    fn from(value: CoverIdsListDataPullMessage) -> Self {
        value.0
    }
}

impl Message for CoverIdsListDataPullMessage {
    type Result = CoverIdsListDataPull;
}

impl Handler<CoverIdsListDataPullMessage> for DirsOptions {
    type Result = CoverIdsListDataPull;
    fn handle(
        &mut self,
        msg: CoverIdsListDataPullMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        CoverIdsListDataPull::new(self.mangas.clone(), msg.into())
    }
}
