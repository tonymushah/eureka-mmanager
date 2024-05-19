pub mod messages;
pub mod task;

use std::collections::HashMap;

use actix::prelude::*;
use mangadex_api::MangaDexClient;
use uuid::Uuid;

use crate::{history::service::HistoryActorService, DirsOptions};

use self::task::MangaDownloadTask;

use super::messages::DropSingleTaskMessage;

#[derive(Debug)]
pub struct MangaDownloadManager {
    dir_option: Addr<DirsOptions>,
    client: MangaDexClient,
    history: Addr<HistoryActorService>,
    tasks: HashMap<Uuid, Addr<MangaDownloadTask>>,
}

impl MangaDownloadManager {
    pub fn new(
        dir_option: Addr<DirsOptions>,
        client: MangaDexClient,
        history: Addr<HistoryActorService>,
    ) -> Self {
        Self {
            dir_option,
            client,
            history,
            tasks: HashMap::new(),
        }
    }
}

impl Actor for MangaDownloadManager {
    type Context = Context<Self>;
}

impl Handler<DropSingleTaskMessage> for MangaDownloadManager {
    type Result = <DropSingleTaskMessage as Message>::Result;
    fn handle(&mut self, msg: DropSingleTaskMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks.remove(&msg.0);
        Ok(())
    }
}
