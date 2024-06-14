pub mod chapter;
pub mod cover;
pub mod manga;

use std::future::Future;

use actix::Addr;
use chapter::images::ChapterImages;
use manga::MangaDeleteData;
use uuid::Uuid;

use crate::{download::state::messages::get::GetManagerStateData, DirsOptions, ManagerCoreResult};

pub use self::{
    chapter::{images::DeleteChapterImagesMessage, DeleteChapterMessage},
    cover::DeleteCoverMessage,
    manga::DeleteMangaMessage,
};

pub trait DeleteDataAsyncTrait: Sync {
    fn delete_chapter_images(
        &self,
        id: Uuid,
        mode: impl Into<ChapterImages> + Send + 'static,
    ) -> impl Future<Output = ManagerCoreResult<()>> + Send;
    fn delete_chapter_images_ignore_conflict(
        &self,
        id: Uuid,
        mode: impl Into<ChapterImages> + Send + 'static,
    ) -> impl Future<Output = ManagerCoreResult<()>> + Send;
    fn delete_chapter(&self, id: Uuid) -> impl Future<Output = ManagerCoreResult<()>> + Send;
    fn delete_cover(&self, id: Uuid) -> impl Future<Output = ManagerCoreResult<()>> + Send;
    fn delete_manga(
        &self,
        id: Uuid,
    ) -> impl Future<Output = ManagerCoreResult<MangaDeleteData>> + Send;
}

impl DeleteDataAsyncTrait for Addr<DirsOptions> {
    async fn delete_chapter_images(
        &self,
        id: Uuid,
        mode: impl Into<ChapterImages> + Send + 'static,
    ) -> ManagerCoreResult<()> {
        self.send(DeleteChapterImagesMessage::new(id, mode)).await?
    }
    async fn delete_chapter_images_ignore_conflict(
        &self,
        id: Uuid,
        mode: impl Into<ChapterImages> + Send + 'static,
    ) -> ManagerCoreResult<()> {
        self.send(DeleteChapterImagesMessage::new(id, mode).ignore_conflict(true))
            .await?
    }
    async fn delete_chapter(&self, id: Uuid) -> ManagerCoreResult<()> {
        self.send(DeleteChapterMessage::new(id)).await?
    }
    async fn delete_cover(&self, id: Uuid) -> ManagerCoreResult<()> {
        self.send(DeleteCoverMessage(id)).await?
    }
    async fn delete_manga(&self, id: Uuid) -> ManagerCoreResult<MangaDeleteData> {
        self.send(DeleteMangaMessage(id)).await?
    }
}

impl<A> DeleteDataAsyncTrait for A
where
    A: GetManagerStateData,
{
    async fn delete_chapter_images(
        &self,
        id: Uuid,
        mode: impl Into<ChapterImages> + Send + 'static,
    ) -> ManagerCoreResult<()> {
        self.get_dir_options()
            .await?
            .delete_chapter_images(id, mode)
            .await
    }
    async fn delete_chapter_images_ignore_conflict(
        &self,
        id: Uuid,
        mode: impl Into<ChapterImages> + Send + 'static,
    ) -> ManagerCoreResult<()> {
        self.get_dir_options()
            .await?
            .delete_chapter_images_ignore_conflict(id, mode)
            .await
    }
    async fn delete_chapter(&self, id: Uuid) -> ManagerCoreResult<()> {
        self.get_dir_options().await?.delete_chapter(id).await
    }
    async fn delete_cover(&self, id: Uuid) -> ManagerCoreResult<()> {
        self.get_dir_options().await?.delete_cover(id).await
    }
    async fn delete_manga(&self, id: Uuid) -> ManagerCoreResult<MangaDeleteData> {
        self.get_dir_options().await?.delete_manga(id).await
    }
}
