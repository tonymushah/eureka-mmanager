pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use self::task::ChapterDownloadTask;

use super::{messages::DropSingleTaskMessage, state::DownloadManagerState};

#[derive(Debug)]
pub struct ChapterDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: HashMap<Uuid, Addr<ChapterDownloadTask>>,
    notify: Arc<Notify>,
}

impl ChapterDownloadManager {
    pub fn new(state: Addr<DownloadManagerState>) -> Self {
        Self {
            state,
            tasks: Default::default(),
            notify: Arc::new(Notify::new()),
        }
    }
}

impl Actor for ChapterDownloadManager {
    type Context = Context<Self>;
}

impl Handler<DropSingleTaskMessage> for ChapterDownloadManager {
    type Result = <DropSingleTaskMessage as Message>::Result;
    fn handle(&mut self, msg: DropSingleTaskMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks.remove(&msg.0);
        self.notify.notify_waiters();
        Ok(())
    }
}
