pub mod cover_data_pull;
pub mod cover_ids_list_data_pull;
pub mod cover_image_data_pull;
pub mod cover_list_data_pull;

use std::{fs::File, future::Future};

use actix::Addr;
use mangadex_api_schema_rust::v5::CoverObject;
use uuid::Uuid;

use crate::{
    data_pulls::cover::{CoverIdsListDataPull, CoverListDataPull},
    download::state::messages::get::GetManagerStateData,
    DirsOptions, MailBoxResult, ManagerCoreResult,
};

pub use self::{
    cover_data_pull::CoverDataPullMessage, cover_ids_list_data_pull::CoverIdsListDataPullMessage,
    cover_image_data_pull::CoverImageDataPullMessage,
    cover_list_data_pull::CoverListDataPullMessage,
};

pub trait CoverDataPullAsyncTrait: Sync {
    fn get_cover(&self, id: Uuid) -> impl Future<Output = ManagerCoreResult<CoverObject>> + Send;
    fn get_cover_image(&self, id: Uuid) -> impl Future<Output = ManagerCoreResult<File>> + Send;
    fn get_covers(&self) -> impl Future<Output = ManagerCoreResult<CoverListDataPull>> + Send;
    fn get_covers_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> impl Future<Output = MailBoxResult<CoverIdsListDataPull>> + Send;
    fn get_cover_with_image(
        &self,
        id: Uuid,
    ) -> impl Future<Output = ManagerCoreResult<(CoverObject, File)>> + Send {
        async move { Ok((self.get_cover(id).await?, self.get_cover_image(id).await?)) }
    }
}

impl CoverDataPullAsyncTrait for Addr<DirsOptions> {
    async fn get_cover(&self, id: Uuid) -> ManagerCoreResult<CoverObject> {
        self.send(CoverDataPullMessage(id)).await?
    }
    async fn get_cover_image(&self, id: Uuid) -> ManagerCoreResult<File> {
        self.send(CoverImageDataPullMessage(id)).await?
    }
    async fn get_covers(&self) -> ManagerCoreResult<CoverListDataPull> {
        self.send(CoverListDataPullMessage).await?
    }
    fn get_covers_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> impl Future<Output = MailBoxResult<CoverIdsListDataPull>> + Send {
        self.send(CoverIdsListDataPullMessage(ids.collect()))
    }
}

impl<A> CoverDataPullAsyncTrait for A
where
    A: GetManagerStateData + Sync,
{
    async fn get_cover(&self, id: Uuid) -> ManagerCoreResult<CoverObject> {
        self.get_dir_options().await?.get_cover(id).await
    }
    async fn get_cover_image(&self, id: Uuid) -> ManagerCoreResult<File> {
        self.get_dir_options().await?.get_cover_image(id).await
    }
    async fn get_covers(&self) -> ManagerCoreResult<CoverListDataPull> {
        self.get_dir_options().await?.get_covers().await
    }
    async fn get_covers_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> MailBoxResult<CoverIdsListDataPull> {
        self.get_dir_options().await?.get_covers_by_ids(ids).await
    }
}
