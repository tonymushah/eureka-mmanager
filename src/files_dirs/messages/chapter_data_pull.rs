use std::{fs::File, io::BufReader};

use actix::prelude::*;
use mangadex_api_schema_rust::v5::{ChapterData, ChapterObject};
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

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
    fn handle(&mut self, msg: ChapterDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        let manga_id_path = self.chapters_add(format!("{}", msg.0)).join("data.json");
        let file = BufReader::new(File::open(manga_id_path)?);
        let manga: ChapterData = serde_json::from_reader(file)?;
        Ok(manga.data)
    }
}
