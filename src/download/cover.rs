pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use self::task::CoverDownloadTask;

use super::{messages::DropSingleTaskMessage, state::DownloadManagerState};

#[derive(Debug)]
pub struct CoverDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: HashMap<Uuid, Addr<CoverDownloadTask>>,
    notify: Arc<Notify>,
}

impl CoverDownloadManager {
    pub fn new(state: Addr<DownloadManagerState>) -> Self {
        Self {
            state,
            tasks: HashMap::new(),
            notify: Arc::new(Notify::new()),
        }
    }
}

impl Actor for CoverDownloadManager {
    type Context = Context<Self>;
}

impl Handler<DropSingleTaskMessage> for CoverDownloadManager {
    type Result = <DropSingleTaskMessage as Message>::Result;
    fn handle(&mut self, msg: DropSingleTaskMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.tasks.remove(&msg.0);
        self.notify.notify_waiters();
        Ok(())
    }
}
