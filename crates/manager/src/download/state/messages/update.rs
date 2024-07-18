pub mod client;
pub mod dir_options;
pub mod history;

use std::future::Future;

use actix::Addr;
use mangadex_api::MangaDexClient;

use crate::{
    download::{messages::state::GetManagerState, state::DownloadManagerState},
    history::service::HistoryActorService,
    DirsOptions, MailBoxResult,
};

pub use self::{
    client::UpdateClientMessage, dir_options::UpdateDirOptionsMessage,
    history::UpdateHistoryMessage,
};

pub trait UpdateManagerStateData: Sync {
    fn set_client(
        &self,
        client: impl Into<MangaDexClient> + Send + 'static,
    ) -> impl Future<Output = MailBoxResult<MangaDexClient>> + Send;
    fn set_dir_options(
        &self,
        dirs_options: impl Into<Addr<DirsOptions>> + Send + 'static,
    ) -> impl Future<Output = MailBoxResult<Addr<DirsOptions>>> + Send;
    fn set_history(
        &self,
        history: impl Into<Addr<HistoryActorService>> + Send + 'static,
    ) -> impl Future<Output = MailBoxResult<Addr<HistoryActorService>>> + Send;
}

impl UpdateManagerStateData for Addr<DownloadManagerState> {
    fn set_client(
        &self,
        client: impl Into<MangaDexClient> + Send + 'static,
    ) -> impl Future<Output = MailBoxResult<MangaDexClient>> + Send {
        self.send(UpdateClientMessage(client.into()))
    }
    fn set_dir_options(
        &self,
        dirs_options: impl Into<Addr<DirsOptions>> + Send + 'static,
    ) -> impl Future<Output = MailBoxResult<Addr<DirsOptions>>> + Send {
        self.send(UpdateDirOptionsMessage(dirs_options.into()))
    }
    fn set_history(
        &self,
        history: impl Into<Addr<HistoryActorService>> + Send + 'static,
    ) -> impl Future<Output = MailBoxResult<Addr<HistoryActorService>>> + Send {
        self.send(UpdateHistoryMessage(history.into()))
    }
}

impl<A> UpdateManagerStateData for A
where
    A: GetManagerState + Sync,
{
    async fn set_client(
        &self,
        client: impl Into<MangaDexClient> + Send + 'static,
    ) -> MailBoxResult<MangaDexClient> {
        self.get_manager_state().await?.set_client(client).await
    }
    async fn set_dir_options(
        &self,
        dirs_options: impl Into<Addr<DirsOptions>> + Send + 'static,
    ) -> MailBoxResult<Addr<DirsOptions>> {
        self.get_manager_state()
            .await?
            .set_dir_options(dirs_options)
            .await
    }
    async fn set_history(
        &self,
        history: impl Into<Addr<HistoryActorService>> + Send + 'static,
    ) -> MailBoxResult<Addr<HistoryActorService>> {
        self.get_manager_state().await?.set_history(history).await
    }
}
