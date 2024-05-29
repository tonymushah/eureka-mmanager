use std::future::Future;

use actix::prelude::*;
use mangadex_api::MangaDexClient;

use crate::{history::service::HistoryActorService, DirsOptions};

use self::{
    chapter::ChapterDownloadManager, cover::CoverDownloadManager, manga::MangaDownloadManager,
    state::DownloadManagerState,
};

pub trait GetManager<T>
where
    T: Actor,
{
    fn get(&self) -> impl Future<Output = Result<Addr<T>, MailboxError>> + Send;
}

pub mod chapter;
pub mod cover;
pub mod manga;
pub mod messages;
pub mod state;

pub struct DownloadManager {
    state: Addr<DownloadManagerState>,
    manga: Addr<MangaDownloadManager>,
    cover: Addr<CoverDownloadManager>,
    chapter: Addr<ChapterDownloadManager>,
}

impl DownloadManager {
    pub async fn new(dir_option: Addr<DirsOptions>, client: MangaDexClient) -> Self {
        let history = HistoryActorService::new(dir_option.clone()).await.start();
        let state = DownloadManagerState::new(dir_option, client, history).start();
        state.into()
    }
}

impl Actor for DownloadManager {
    type Context = Context<Self>;
}

impl From<Addr<DownloadManagerState>> for DownloadManager {
    fn from(state: Addr<DownloadManagerState>) -> Self {
        Self {
            manga: MangaDownloadManager::new(state.clone()).start(),
            cover: CoverDownloadManager::new(state.clone()).start(),
            chapter: ChapterDownloadManager::new(state.clone()).start(),
            state,
        }
    }
}
