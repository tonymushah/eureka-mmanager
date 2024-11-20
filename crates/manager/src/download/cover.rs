pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use self::task::CoverDownloadTask;

use super::{
    messages::{DropSingleTaskMessage, StartDownload},
    state::{DownloadManagerState, DownloadMessageState},
    traits::{managers::TaskManager, task::AsyncState},
};

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

#[derive(Debug, Clone, Copy)]
pub struct CoverDownloadMessage {
    id: Uuid,
    state: DownloadMessageState,
}

impl CoverDownloadMessage {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            state: DownloadMessageState::Pending,
        }
    }
    pub fn state(self, state: DownloadMessageState) -> Self {
        Self { state, ..self }
    }
}

impl From<Uuid> for CoverDownloadMessage {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl From<CoverDownloadMessage> for Uuid {
    fn from(value: CoverDownloadMessage) -> Self {
        value.id
    }
}

impl Message for CoverDownloadMessage {
    type Result = Addr<CoverDownloadTask>;
}

impl TaskManager for CoverDownloadManager {
    type Task = CoverDownloadTask;
    type DownloadMessage = CoverDownloadMessage;

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
            .or_insert_with(|| CoverDownloadTask::new(msg.id, ctx.address()).start())
            .clone();
        let re_task = task.clone();
        self.notify.notify_waiters();

        if let DownloadMessageState::Downloading = msg.state {
            let fut = async move {
                let state = re_task.state().await?;
                if !state.is_loading() {
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

impl Handler<CoverDownloadMessage> for CoverDownloadManager {
    type Result = <CoverDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: CoverDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        self.new_task(msg, ctx)
    }
}

impl Handler<DropSingleTaskMessage> for CoverDownloadManager {
    type Result = <DropSingleTaskMessage as Message>::Result;
    fn handle(&mut self, msg: DropSingleTaskMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.drop_task(msg.0);
    }
}
