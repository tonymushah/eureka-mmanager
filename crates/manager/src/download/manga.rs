pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use self::task::MangaDownloadTask;

use super::{
    messages::{DropSingleTaskMessage, StartDownload},
    state::{DownloadManagerState, DownloadMessageState, TaskState},
    traits::{managers::TaskManager, task::AsyncState},
};

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

#[derive(Debug, Clone, Copy)]
pub struct MangaDownloadMessage {
    id: Uuid,
    // TODO Add cover_art download support
    state: DownloadMessageState,
}

impl From<Uuid> for MangaDownloadMessage {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl From<MangaDownloadMessage> for Uuid {
    fn from(value: MangaDownloadMessage) -> Self {
        value.id
    }
}

impl MangaDownloadMessage {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            state: Default::default(),
        }
    }
    pub fn state(self, state: DownloadMessageState) -> Self {
        Self { state, ..self }
    }
}

impl Message for MangaDownloadMessage {
    type Result = Addr<MangaDownloadTask>;
}

impl TaskManager for MangaDownloadManager {
    type DownloadMessage = MangaDownloadMessage;
    type Task = MangaDownloadTask;
    fn drop_task(&mut self, id: Uuid) {
        self.tasks.remove(&id);
        self.notify.notify_waiters();
    }
    fn state(&self) -> Addr<DownloadManagerState> {
        self.state.clone()
    }
    fn notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }
    fn tasks_id(&self) -> Vec<Uuid> {
        self.tasks.keys().copied().collect()
    }

    fn tasks(&self) -> Vec<Addr<Self::Task>> {
        self.tasks
            .values() /* .filter(|v| v.connected())*/
            .cloned()
            .collect()
    }
    fn new_task(
        &mut self,
        msg: Self::DownloadMessage,
        ctx: &mut Self::Context,
    ) -> Addr<Self::Task> {
        let task = self
            .tasks
            .entry(msg.id)
            .or_insert_with(|| MangaDownloadTask::new(msg.id, ctx.address()).start())
            .clone();
        let re_task = task.clone();
        self.notify.notify_waiters();

        if let DownloadMessageState::Downloading = msg.state {
            let fut = async move { re_task.state().await.map(|s| (s, re_task)) }
                .into_actor(self)
                .map_ok(move |(s, re_task), _this, _ctx| {
                    if s != TaskState::Loading {
                        re_task.do_send(StartDownload);
                    }
                })
                .map(|s, _, _| {
                    let _ = s;
                });
            ctx.wait(fut)
        }
        task
    }
}

impl Handler<MangaDownloadMessage> for MangaDownloadManager {
    type Result = <MangaDownloadMessage as Message>::Result;
    // TODO Add support for the DownloadState
    fn handle(&mut self, msg: MangaDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        self.new_task(msg, ctx)
    }
}

impl Handler<DropSingleTaskMessage> for MangaDownloadManager {
    type Result = <DropSingleTaskMessage as Message>::Result;
    fn handle(&mut self, msg: DropSingleTaskMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.drop_task(msg.0);
    }
}
