pub mod chapter_at_home_pull;
pub mod chapter_data_pull;
pub mod chapter_ids_list_data_pull;
pub mod chapter_image_data_pull;
pub mod chapter_image_data_saver_pull;
pub mod chapter_list_data_pull;

use std::{fs::File, future::Future, path::Path};

use actix::Addr;
use mangadex_api_schema_rust::v5::ChapterObject;
use uuid::Uuid;

use crate::{
    data_pulls::chapter::{
        ids::ChapterIdsListDataPull, images::ChapterImagesData, list::ChapterListDataPull,
    },
    download::state::messages::get::GetManagerStateData,
    DirsOptions, MailBoxResult, ManagerCoreResult,
};

pub use self::{
    chapter_at_home_pull::ChapterImagesPullMessage, chapter_data_pull::ChapterDataPullMessage,
    chapter_ids_list_data_pull::ChapterIdsListDataPullMessage,
    chapter_image_data_pull::ChapterImageDataPullMessage,
    chapter_image_data_saver_pull::ChapterImageDataSaverPullMessage,
    chapter_list_data_pull::ChapterListDataPullMessage,
};

pub trait ChapterDataPullAsyncTrait: Sync {
    fn get_chapter(
        &self,
        id: Uuid,
    ) -> impl Future<Output = ManagerCoreResult<ChapterObject>> + Send;
    fn get_chapter_images(
        &self,
        id: Uuid,
    ) -> impl Future<Output = ManagerCoreResult<ChapterImagesData>> + Send;
    fn get_chapters(&self) -> impl Future<Output = ManagerCoreResult<ChapterListDataPull>> + Send;
    fn get_chapter_image(
        &self,
        id: Uuid,
        filename: impl AsRef<Path> + Send + 'static,
    ) -> impl Future<Output = ManagerCoreResult<File>> + Send;
    fn get_chapter_image_data_saver(
        &self,
        id: Uuid,
        filename: impl AsRef<Path> + Send + 'static,
    ) -> impl Future<Output = ManagerCoreResult<File>> + Send;
    fn get_chapters_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> impl Future<Output = MailBoxResult<ChapterIdsListDataPull>> + Send;
}

impl ChapterDataPullAsyncTrait for Addr<DirsOptions> {
    async fn get_chapter(&self, id: Uuid) -> ManagerCoreResult<ChapterObject> {
        self.send(ChapterDataPullMessage(id)).await?
    }
    async fn get_chapter_images(&self, id: Uuid) -> ManagerCoreResult<ChapterImagesData> {
        self.send(ChapterImagesPullMessage(id)).await?
    }
    async fn get_chapters(&self) -> ManagerCoreResult<ChapterListDataPull> {
        self.send(ChapterListDataPullMessage).await?
    }
    async fn get_chapter_image(
        &self,
        id: Uuid,
        filename: impl AsRef<Path> + Send + 'static,
    ) -> ManagerCoreResult<File> {
        self.send(ChapterImageDataPullMessage(id, filename)).await?
    }
    async fn get_chapter_image_data_saver(
        &self,
        id: Uuid,
        filename: impl AsRef<Path> + Send + 'static,
    ) -> ManagerCoreResult<File> {
        self.send(ChapterImageDataSaverPullMessage(id, filename))
            .await?
    }
    fn get_chapters_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid>,
    ) -> impl Future<Output = MailBoxResult<ChapterIdsListDataPull>> + Send {
        self.send(ChapterIdsListDataPullMessage(ids.collect()))
    }
}

impl<A> ChapterDataPullAsyncTrait for A
where
    A: GetManagerStateData + Sync,
{
    async fn get_chapter(&self, id: Uuid) -> ManagerCoreResult<ChapterObject> {
        self.get_dir_options().await?.get_chapter(id).await
    }

    async fn get_chapter_images(&self, id: Uuid) -> ManagerCoreResult<ChapterImagesData> {
        self.get_dir_options().await?.get_chapter_images(id).await
    }

    async fn get_chapters(&self) -> ManagerCoreResult<ChapterListDataPull> {
        self.get_dir_options().await?.get_chapters().await
    }

    async fn get_chapter_image(
        &self,
        id: Uuid,
        filename: impl AsRef<Path> + Send + 'static,
    ) -> ManagerCoreResult<File> {
        self.get_dir_options()
            .await?
            .get_chapter_image(id, filename)
            .await
    }

    async fn get_chapter_image_data_saver(
        &self,
        id: Uuid,
        filename: impl AsRef<Path> + Send + 'static,
    ) -> ManagerCoreResult<File> {
        self.get_dir_options()
            .await?
            .get_chapter_image_data_saver(id, filename)
            .await
    }

    async fn get_chapters_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> MailBoxResult<ChapterIdsListDataPull> {
        self.get_dir_options().await?.get_chapters_by_ids(ids).await
    }
}
