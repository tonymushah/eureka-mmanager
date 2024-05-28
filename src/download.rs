use std::future::Future;

use actix::prelude::*;
use mangadex_api::MangaDexClient;

use crate::{history::service::HistoryActorService, DirsOptions};

use self::{cover::CoverDownloadManager, manga::MangaDownloadManager, state::DownloadManagerState};

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
}

impl DownloadManager {
    pub fn new(
        dir_option: Addr<DirsOptions>,
        client: MangaDexClient,
        history: Addr<HistoryActorService>,
    ) -> Self {
        let state = DownloadManagerState::new(dir_option, client, history).start();
        {
            Self {
                manga: MangaDownloadManager::new(state.clone()).start(),
                cover: CoverDownloadManager::new(state.clone()).start(),
                state,
            }
        }
    }
}

impl Actor for DownloadManager {
    type Context = Context<Self>;
}
