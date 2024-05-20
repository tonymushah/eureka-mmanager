pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use self::task::MangaDownloadTask;

use super::{messages::DropSingleTaskMessage, state::DownloadManagerState};

#[derive(Debug)]
pub struct MangaDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: HashMap<Uuid, Addr<MangaDownloadTask>>,
    notify: Arc<Notify>,
}

impl MangaDownloadManager {
    pub fn new(state: Addr<DownloadManagerState>) -> Self {
        Self {
            state,
            tasks: HashMap::new(),
            notify: Arc::new(Notify::new()),
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
        self.notify.notify_waiters();
        Ok(())
    }
}
