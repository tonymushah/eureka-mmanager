pub mod client;
pub mod dir_options;
pub mod history;

use std::future::Future;

use actix::Addr;
use mangadex_api::MangaDexClient;

use crate::{
    download::state::DownloadManagerState, history::service::HistoryActorService, DirsOptions,
    MailBoxResult,
};

pub use self::{
    client::GetClientMessage, dir_options::GetDirsOptionsMessage, history::GetHistoryMessage,
};

pub trait GetManagerStateData: Sync {
    fn client(&self) -> impl Future<Output = MailBoxResult<MangaDexClient>> + Send;
    fn dir_options(&self) -> impl Future<Output = MailBoxResult<Addr<DirsOptions>>> + Send;
    fn history(&self) -> impl Future<Output = MailBoxResult<Addr<HistoryActorService>>> + Send;
}

impl GetManagerStateData for Addr<DownloadManagerState> {
    fn client(&self) -> impl Future<Output = MailBoxResult<MangaDexClient>> + Send {
        self.send(GetClientMessage)
    }
    fn dir_options(&self) -> impl Future<Output = MailBoxResult<Addr<DirsOptions>>> + Send {
        self.send(GetDirsOptionsMessage)
    }
    fn history(&self) -> impl Future<Output = MailBoxResult<Addr<HistoryActorService>>> + Send {
        self.send(GetHistoryMessage)
    }
}
