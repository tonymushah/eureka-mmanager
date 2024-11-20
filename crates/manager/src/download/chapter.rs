pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use task::DownloadMode;
use tokio::sync::Notify;
use uuid::Uuid;

use self::task::ChapterDownloadTask;

use super::{
    messages::{DropSingleTaskMessage, StartDownload},
    state::{DownloadManagerState, DownloadMessageState},
    traits::{managers::TaskManager, task::AsyncState},
};

#[derive(Debug)]
pub struct ChapterDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: HashMap<Uuid, Addr<ChapterDownloadTask>>,
    notify: Arc<Notify>,
}

#[derive(Debug, Clone, Copy)]
pub struct ChapterDownloadMessage {
    id: Uuid,
    state: DownloadMessageState,
    mode: DownloadMode,
}

impl ChapterDownloadMessage {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            state: DownloadMessageState::Pending,
            mode: DownloadMode::Normal,
        }
    }
    pub fn state(self, state: DownloadMessageState) -> Self {
        Self { state, ..self }
    }
    pub fn mode<M: Into<DownloadMode>>(self, mode: M) -> Self {
        Self {
            mode: mode.into(),
            ..self
        }
    }
}

impl From<Uuid> for ChapterDownloadMessage {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl From<ChapterDownloadMessage> for Uuid {
    fn from(value: ChapterDownloadMessage) -> Self {
        value.id
    }
}

impl Message for ChapterDownloadMessage {
    type Result = Addr<<ChapterDownloadManager as TaskManager>::Task>;
}

impl TaskManager for ChapterDownloadManager {
    type Task = ChapterDownloadTask;
    type DownloadMessage = ChapterDownloadMessage;
    fn state(&self) -> Addr<DownloadManagerState> {
        self.state.clone()
    }
    fn notify(&self) -> Arc<Notify> {
        self.notify.clone()
    }
    fn tasks(&self) -> Vec<Addr<Self::Task>> {
        self.tasks.values().cloned().collect()
    }
    fn tasks_id(&self) -> Vec<Uuid> {
        self.tasks.keys().copied().collect()
    }
    fn new_task(
        &mut self,
        msg: Self::DownloadMessage,
        ctx: &mut Self::Context,
    ) -> Addr<Self::Task> {
        let task = self
            .tasks
            .entry(msg.id)
            .or_insert_with(|| Self::Task::new(msg.id, msg.mode, ctx.address()).start())
            .clone();
        let re_task = task.clone();
        self.notify.notify_waiters();

        if let DownloadMessageState::Downloading = msg.state {
            let fut = async move {
                let state = re_task.state().await?;
                if !state.is_loading() {
                    re_task.send(msg.mode).await?;
                    re_task.send(StartDownload).await?;
                }
                Ok::<_, actix::MailboxError>(())
            }
            .into_actor(self)
            .map(|s, _, _| {
                if let Err(err) = s {
                    log::error!("{err}");
                }
            });
            ctx.wait(fut)
        }
        task
    }

    fn drop_task(&mut self, id: Uuid) {
        self.tasks.remove(&id);
        self.notify.notify_waiters();
    }
}

impl Handler<ChapterDownloadMessage> for ChapterDownloadManager {
    type Result = <ChapterDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: ChapterDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        self.new_task(msg, ctx)
    }
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
        self.drop_task(msg.0);
    }
}
