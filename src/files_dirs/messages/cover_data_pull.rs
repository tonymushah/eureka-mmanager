use std::{fs::File, io::BufReader};

use actix::prelude::*;
use mangadex_api_schema_rust::v5::{CoverData, CoverObject};
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

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
    // TODO add cbor support
    fn handle(&mut self, msg: CoverDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        let manga_id_path = self.covers_add(format!("{}.json", msg.0));
        let file = BufReader::new(File::open(manga_id_path)?);
        let manga: CoverData = serde_json::from_reader(file)?;
        Ok(manga.data)
    }
}
