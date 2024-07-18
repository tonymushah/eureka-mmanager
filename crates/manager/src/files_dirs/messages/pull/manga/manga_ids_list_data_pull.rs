use std::collections::HashMap;

use actix::prelude::*;
use mangadex_api_types_rust::{Language, RelationshipType};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{
    data_pulls::manga::ids::MangaIdsListDataPull,
    files_dirs::messages::pull::chapter::ChapterListDataPullMessage, DirsOptions,
};

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
    type Result = ResponseFuture<MangaIdsListDataPull>;
    fn handle(
        &mut self,
        msg: MangaIdsListDataPullMessage,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        let msg: Vec<Uuid> = msg.into();
        let pull = self.pull_mangas_ids(msg.clone());
        let chap_pull = self.handle(ChapterListDataPullMessage, ctx).map(move |p| {
            StreamExt::filter_map(p, move |chapter| {
                let manga = chapter
                    .find_first_relationships(RelationshipType::Manga)?
                    .id;
                if msg.contains(&manga) {
                    Some(chapter)
                } else {
                    None
                }
            })
            .fold(HashMap::<Uuid, Vec<Language>>::new(), |mut acc, value| {
                if let Some(manga_id) = value
                    .find_first_relationships(RelationshipType::Manga)
                    .map(|rel| rel.id)
                {
                    let current_lang = value.attributes.translated_language;
                    let langs = acc.entry(manga_id).or_default();
                    if !langs.contains(&current_lang) {
                        langs.push(current_lang);
                    }
                }
                acc
            })
        });
        Box::pin(async move {
            let mut pull = pull;
            if let Ok(langs_fut) = chap_pull {
                pull = pull.with_available_langs(langs_fut.await);
            }
            pull
        })
    }
}
