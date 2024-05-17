use std::{
    fs::remove_file,
    sync::{Arc, Mutex},
};

use actix::prelude::*;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{
    data_pulls::{
        chapter::ChapterListDataPullFilterParams, cover::filter::CoverListDataPullFilterParams,
        IntoParamedFilteredStream,
    },
    files_dirs::messages::pull::{
        chapter::ChapterListDataPullMessage, cover::CoverListDataPullMessage,
    },
    DirsOptions,
};

use super::{DeleteChapterMessage, DeleteCoverMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeleteMangaMessage(pub Uuid);

impl Message for DeleteMangaMessage {
    type Result = crate::ManagerCoreResult<()>;
}

impl From<DeleteMangaMessage> for Uuid {
    fn from(value: DeleteMangaMessage) -> Self {
        value.0
    }
}

impl From<Uuid> for DeleteMangaMessage {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

#[derive(Debug, Default)]
struct MangaDeleteData {
    chapters: Vec<Uuid>,
    covers: Vec<Uuid>,
}

impl Handler<DeleteMangaMessage> for DirsOptions {
    type Result = <DeleteMangaMessage as Message>::Result;
    fn handle(&mut self, msg: DeleteMangaMessage, ctx: &mut Self::Context) -> Self::Result {
        let manga_path = self.mangas_add(format!("{}.json", msg.0));
        let manga_chapters_data_pull = self
            .handle(ChapterListDataPullMessage, ctx)?
            .to_filtered(ChapterListDataPullFilterParams {
                manga_id: Some(msg.0),
                ..Default::default()
            })
            .map(|c| c.id);
        let manga_covers_data_pull = self
            .handle(CoverListDataPullMessage, ctx)?
            .to_filtered(CoverListDataPullFilterParams {
                manga_ids: vec![msg.0],
                ..Default::default()
            })
            .map(|c| c.id);
        let data = Arc::new(Mutex::<MangaDeleteData>::default());
        let data_wait = data.clone();
        let pull_task = async move {
            let chapters = manga_chapters_data_pull.collect().await;
            let covers = manga_covers_data_pull.collect().await;
            if let Ok(mut data) = data_wait.lock() {
                data.chapters = chapters;
                data.covers = covers;
            }
        }
        .into_actor(self);
        ctx.wait(pull_task);
        if let Ok(delete_data) = data.lock() {
            for chapter in &delete_data.chapters {
                if let Err(e) = self.handle(DeleteChapterMessage::new(*chapter), ctx) {
                    log::error!("{e}");
                }
            }
            for cover in &delete_data.covers {
                if let Err(e) = self.handle(DeleteCoverMessage(*cover), ctx) {
                    log::error!("{e}");
                }
            }
        }
        remove_file(manga_path)?;
        Ok(())
    }
}
