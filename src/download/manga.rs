pub mod messages;
pub mod task;

use std::collections::HashMap;

use actix::prelude::*;
use uuid::Uuid;

use self::task::MangaDownloadTask;

use super::{messages::DropSingleTaskMessage, state::DownloadManagerState};

#[derive(Debug)]
pub struct MangaDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: HashMap<Uuid, Addr<MangaDownloadTask>>,
}

impl MangaDownloadManager {
    pub fn new(state: Addr<DownloadManagerState>) -> Self {
        Self {
            state,
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
