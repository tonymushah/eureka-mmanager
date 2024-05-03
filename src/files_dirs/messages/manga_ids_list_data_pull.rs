use actix::prelude::*;
use uuid::Uuid;

use crate::{data_pulls::manga::list::ids::MangaIdsListDataPull, DirsOptions};

#[derive(Debug, Clone, Hash, Default)]
pub struct MangaIdsListDataPullMessage(pub Vec<Uuid>);

impl From<Vec<Uuid>> for MangaIdsListDataPullMessage {
    fn from(value: Vec<Uuid>) -> Self {
        Self(value)
    }
}

impl From<MangaIdsListDataPullMessage> for Vec<Uuid> {
    fn from(value: MangaIdsListDataPullMessage) -> Self {
        value.0
    }
}

impl Message for MangaIdsListDataPullMessage {
    type Result = MangaIdsListDataPull;
}

impl Handler<MangaIdsListDataPullMessage> for DirsOptions {
    type Result = MangaIdsListDataPull;
    fn handle(
        &mut self,
        msg: MangaIdsListDataPullMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        MangaIdsListDataPull::new(self.mangas.clone(), msg.into())
    }
}
