pub mod manga_data_pull;
pub mod manga_ids_list_data_pull;
pub mod manga_list_data_pull;

use std::future::Future;

use actix::Addr;
use mangadex_api_schema_rust::v5::MangaObject;
use uuid::Uuid;

use crate::{
    data_pulls::manga::{MangaIdsListDataPull, MangaListDataPull},
    download::state::messages::get::GetManagerStateData,
    DirsOptions, MailBoxResult, ManagerCoreResult,
};

pub use self::{
    manga_data_pull::MangaDataPullMessage, manga_ids_list_data_pull::MangaIdsListDataPullMessage,
    manga_list_data_pull::MangaListDataPullMessage,
};

pub trait MangaDataPullAsyncTrait: Sync {
    fn get_manga(&self, id: Uuid) -> impl Future<Output = ManagerCoreResult<MangaObject>> + Send;
    fn get_manga_list(&self) -> impl Future<Output = ManagerCoreResult<MangaListDataPull>> + Send;
    fn get_manga_list_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> impl Future<Output = MailBoxResult<MangaIdsListDataPull>> + Send;
}

impl MangaDataPullAsyncTrait for Addr<DirsOptions> {
    async fn get_manga(&self, id: Uuid) -> ManagerCoreResult<MangaObject> {
        self.send(MangaDataPullMessage(id)).await?
    }
    async fn get_manga_list(&self) -> ManagerCoreResult<MangaListDataPull> {
        self.send(MangaListDataPullMessage).await?
    }
    fn get_manga_list_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> impl Future<Output = MailBoxResult<MangaIdsListDataPull>> {
        self.send(MangaIdsListDataPullMessage(ids.collect()))
    }
}

impl<A> MangaDataPullAsyncTrait for A
where
    A: GetManagerStateData + Sync,
{
    async fn get_manga(&self, id: Uuid) -> ManagerCoreResult<MangaObject> {
        self.get_dir_options().await?.get_manga(id).await
    }
    async fn get_manga_list(&self) -> ManagerCoreResult<MangaListDataPull> {
        self.get_dir_options().await?.get_manga_list().await
    }
    async fn get_manga_list_by_ids(
        &self,
        ids: impl Iterator<Item = Uuid> + Send,
    ) -> MailBoxResult<MangaIdsListDataPull> {
        self.get_dir_options()
            .await?
            .get_manga_list_by_ids(ids)
            .await
    }
}
