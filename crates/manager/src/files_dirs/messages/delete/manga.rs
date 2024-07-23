use std::fs::remove_file;

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
    type Result = crate::ManagerCoreResult<MangaDeleteData>;
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

#[derive(Debug, Default, Clone)]
pub struct MangaDeleteData {
    pub chapters: Vec<Uuid>,
    pub covers: Vec<Uuid>,
}

impl Handler<DeleteMangaMessage> for DirsOptions {
    type Result = ResponseActFuture<Self, <DeleteMangaMessage as Message>::Result>;
    fn handle(&mut self, msg: DeleteMangaMessage, ctx: &mut Self::Context) -> Self::Result {
        let manga_path = self.mangas_add(format!("{}.json", msg.0));
        let manga_chapters_data_pull = {
            let mut to_fn = || {
                Ok::<_, crate::Error>(
                    self.handle(ChapterListDataPullMessage, ctx)?
                        .to_filtered(ChapterListDataPullFilterParams {
                            manga_ids: vec![msg.0],
                            ..Default::default()
                        })
                        .map(|c| c.id)
                        .collect::<Vec<Uuid>>(),
                )
            };
            to_fn()
        };
        let manga_covers_data_pull = {
            let mut to_fn = || {
                Ok::<_, crate::Error>(
                    self.handle(CoverListDataPullMessage, ctx)?
                        .to_filtered(CoverListDataPullFilterParams {
                            manga_ids: vec![msg.0],
                            ..Default::default()
                        })
                        .map(|c| c.id)
                        .collect::<Vec<Uuid>>(),
                )
            };
            to_fn()
        };
        let fut = async move {
            Ok::<_, crate::Error>(MangaDeleteData {
                chapters: manga_chapters_data_pull?.await,
                covers: manga_covers_data_pull?.await,
            })
        }
        .into_actor(self)
        .map_ok(move |delete_data, this, ctx| {
            for chapter in &delete_data.chapters {
                if let Err(e) = this.handle(DeleteChapterMessage::new(*chapter), ctx) {
                    log::error!("{e}");
                }
            }
            for cover in &delete_data.covers {
                if let Err(e) = this.handle(DeleteCoverMessage(*cover), ctx) {
                    log::error!("{e}");
                }
            }
            if let Err(e) = remove_file(manga_path) {
                log::error!("{e}");
            }
            delete_data
        });
        Box::pin(fut)
    }
}
