use actix::{Actor, Addr};
use mangadex_api::MangaDexClient;

use crate::{history::service::HistoryActorService, DirsOptions};

#[derive(Debug)]
pub struct DownloadManager {
    dir_option: Addr<DirsOptions>,
    client: MangaDexClient,
    history: Addr<HistoryActorService>,
}

impl DownloadManager {
    pub async fn new(dir_option: Addr<DirsOptions>, client: MangaDexClient) -> Self {
        let history = HistoryActorService::new(dir_option.clone()).await.start();
        Self {
            dir_option,
            client,
            history,
        }
    }
}
