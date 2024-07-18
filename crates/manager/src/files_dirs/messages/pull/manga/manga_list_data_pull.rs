use std::collections::HashMap;

use actix::prelude::*;
use mangadex_api_types_rust::{Language, RelationshipType};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{
    data_pulls::manga::list::MangaListDataPull,
    files_dirs::messages::pull::chapter::ChapterListDataPullMessage, DirsOptions,
    ManagerCoreResult,
};

#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct MangaListDataPullMessage;

impl Message for MangaListDataPullMessage {
    type Result = ManagerCoreResult<MangaListDataPull>;
}

impl Handler<MangaListDataPullMessage> for DirsOptions {
    type Result = ResponseFuture<ManagerCoreResult<MangaListDataPull>>;
    fn handle(&mut self, _msg: MangaListDataPullMessage, ctx: &mut Self::Context) -> Self::Result {
        let pull = self.pull_all_mangas();
        let chap_pull = self.handle(ChapterListDataPullMessage, ctx).map(|p| {
            StreamExt::fold(
                p,
                HashMap::<Uuid, Vec<Language>>::new(),
                |mut acc, value| {
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
                },
            )
        });
        Box::pin(async move {
            let mut pull = pull?;
            if let Ok(langs_fut) = chap_pull {
                pull = pull.with_available_langs(langs_fut.await);
            }
            Ok(pull)
        })
    }
}
