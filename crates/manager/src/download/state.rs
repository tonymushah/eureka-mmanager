pub mod messages;
pub mod task;

use crate::{history::service::HistoryActorService, DirsOptions};

pub use self::task::*;

use actix::prelude::*;
use mangadex_api::MangaDexClient;

#[derive(Debug)]
pub struct DownloadManagerState {
    dir_option: Addr<DirsOptions>,
    client: MangaDexClient,
    history: Addr<HistoryActorService>,
}

impl DownloadManagerState {
    pub fn new(
        dir_option: Addr<DirsOptions>,
        client: MangaDexClient,
        history: Addr<HistoryActorService>,
    ) -> Self {
        Self {
            dir_option,
            client,
            history,
        }
    }
}

impl Actor for DownloadManagerState {
    type Context = Context<Self>;
}
