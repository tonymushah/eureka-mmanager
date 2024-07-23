use std::ops::Deref;

use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::Language;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{
    data_pulls::Pull,
    files_dirs::messages::pull::chapter::ChapterListDataPullMessage,
    prelude::{ChapterListDataPullFilterParams, IntoParamedFilteredStream},
    DirsOptions, ManagerCoreResult,
};

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
    type Result = ResponseFuture<<MangaDataPullMessage as Message>::Result>;

    fn handle(&mut self, msg: MangaDataPullMessage, ctx: &mut Self::Context) -> Self::Result {
        let manga: ManagerCoreResult<MangaObject> = self
            .deref()
            .deref()
            .pull(msg.into())
            .map_err(|e: api_core::Error| e.into());
        let chapter_pull = self.handle(ChapterListDataPullMessage, ctx);
        Box::pin(async move {
            let mut manga = manga?;
            if let Ok(pull) = chapter_pull {
                let langs = pull
                    .to_filtered(ChapterListDataPullFilterParams {
                        manga_ids: vec![manga.id],
                        ..Default::default()
                    })
                    .fold(Vec::<Language>::new(), |mut acc, chapter| {
                        let lang = chapter.attributes.translated_language;
                        if !acc.contains(&lang) {
                            acc.push(lang);
                        }
                        acc
                    })
                    .await;
                manga.attributes.available_translated_languages = langs;
            }
            Ok(manga)
        })
    }
}
