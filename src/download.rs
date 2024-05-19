use actix::prelude::*;
use mangadex_api::MangaDexClient;

use crate::{history::service::HistoryActorService, DirsOptions};

use self::manga::MangaDownloadManager;

pub mod manga;
pub mod messages;
pub mod state;

pub struct DownloadManager {
    manga: Addr<MangaDownloadManager>,
}

impl DownloadManager {
    pub fn new(
        dir_option: Addr<DirsOptions>,
        client: MangaDexClient,
        history: Addr<HistoryActorService>,
    ) -> Self {
        {
            Self {
                manga: MangaDownloadManager::new(dir_option, client, history).start(),
            }
        }
    }
}

impl Actor for DownloadManager {
    type Context = Context<Self>;
}
