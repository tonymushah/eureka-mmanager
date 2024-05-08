use std::{fs::File, io::BufReader};

use actix::prelude::*;
use mangadex_api_schema_rust::v5::{MangaData, MangaObject};
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

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
    // TODO add cbor support
    fn handle(&mut self, msg: MangaDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        let manga_id_path = self.mangas_add(format!("{}.json", msg.0));
        let file = BufReader::new(File::open(manga_id_path)?);
        let manga: MangaData = serde_json::from_reader(file)?;
        Ok(manga.data)
    }
}
