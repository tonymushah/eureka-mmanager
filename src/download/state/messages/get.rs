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
    client::GetClientMessage, dir_options::GetDirsOptionsMessage, history::GetHistoryMessage,
};

pub trait GetManagerStateData: Sync {
    fn get_client(&self) -> impl Future<Output = MailBoxResult<MangaDexClient>> + Send;
    fn get_dir_options(&self) -> impl Future<Output = MailBoxResult<Addr<DirsOptions>>> + Send;
    fn get_history(&self) -> impl Future<Output = MailBoxResult<Addr<HistoryActorService>>> + Send;
}

impl GetManagerStateData for Addr<DownloadManagerState> {
    fn get_client(&self) -> impl Future<Output = MailBoxResult<MangaDexClient>> + Send {
        self.send(GetClientMessage)
    }
    fn get_dir_options(&self) -> impl Future<Output = MailBoxResult<Addr<DirsOptions>>> + Send {
        self.send(GetDirsOptionsMessage)
    }
    fn get_history(&self) -> impl Future<Output = MailBoxResult<Addr<HistoryActorService>>> + Send {
        self.send(GetHistoryMessage)
    }
}

impl<A> GetManagerStateData for A
where
    A: GetManagerState + Sync,
{
    async fn get_client(&self) -> MailBoxResult<MangaDexClient> {
        self.get_manager_state().await?.get_client().await
    }
    async fn get_dir_options(&self) -> MailBoxResult<Addr<DirsOptions>> {
        self.get_manager_state().await?.get_dir_options().await
    }
    async fn get_history(&self) -> MailBoxResult<Addr<HistoryActorService>> {
        self.get_manager_state().await?.get_history().await
    }
}
