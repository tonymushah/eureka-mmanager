pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::{WeakAddr, prelude::*};
use task::DownloadMode;
use tokio::sync::Notify;
use uuid::Uuid;

use crate::download::messages::StopTask;

use self::task::ChapterDownloadTask;

use super::{
    messages::{DropSingleTaskMessage, GetTaskMessage, StartDownload},
    state::{DownloadManagerState, DownloadMessageState},
    traits::{managers::TaskManager, task::AsyncState},
};

#[derive(Debug)]
pub struct ChapterDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: HashMap<Uuid, WeakAddr<ChapterDownloadTask>>,
    notify: Arc<Notify>,
}

#[derive(Debug, Clone, Copy)]
pub struct ChapterDownloadMessage {
    id: Uuid,
    state: DownloadMessageState,
    mode: DownloadMode,
    force_port_443: bool,
}

impl ChapterDownloadMessage {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            state: DownloadMessageState::Pending,
            mode: DownloadMode::Normal,
            force_port_443: false,
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
    pub fn force_port_443(self, force_port_443: bool) -> Self {
        Self {
            force_port_443,
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
        self.tasks
            .values()
            .flat_map(|task| task.upgrade())
            .collect()
    }
    fn tasks_id(&self) -> Vec<Uuid> {
        self.tasks
            .iter()
            .flat_map(|(id, tasks)| {
                if tasks.upgrade().is_some() {
                    Some(id)
                } else {
                    None
                }
            })
            .copied()
            .collect()
    }
    fn new_task(
        &mut self,
        msg: Self::DownloadMessage,
        ctx: &mut Self::Context,
    ) -> Addr<Self::Task> {
        let task = {
            match self.tasks.entry(msg.id) {
                std::collections::hash_map::Entry::Occupied(mut occupied_entry) => {
                    let weak = occupied_entry.get_mut();
                    if let Some(tsk) = weak.upgrade() {
                        tsk
                    } else {
                        let tsk =
                            Self::Task::new(msg.id, msg.mode, msg.force_port_443, ctx.address())
                                .start();
                        let _weak = std::mem::replace(weak, tsk.downgrade());
                        tsk
                    }
                }
                std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                    let tsk = Self::Task::new(msg.id, msg.mode, msg.force_port_443, ctx.address())
                        .start();
                    vacant_entry.insert(tsk.downgrade());
                    tsk
                }
            }
        };
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
        if let Some(task) = self.tasks.get(&id)
            && task.upgrade().is_none()
        {
            self.tasks.remove(&id);
        }
        self.notify.notify_waiters();
    }
    fn get_task(&self, id: Uuid) -> Option<Addr<Self::Task>> {
        self.tasks.get(&id).and_then(WeakAddr::upgrade)
    }
}

impl Handler<ChapterDownloadMessage> for ChapterDownloadManager {
    type Result = <ChapterDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: ChapterDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        self.new_task(msg, ctx)
    }
}

impl Drop for ChapterDownloadManager {
    fn drop(&mut self) {
        self.tasks
            .values()
            .flat_map(|maybe_task| maybe_task.upgrade())
            .for_each(|task| task.do_send(StopTask));
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

impl Handler<GetTaskMessage<ChapterDownloadTask>> for ChapterDownloadManager {
    type Result = <GetTaskMessage<ChapterDownloadTask> as Message>::Result;
    fn handle(
        &mut self,
        msg: GetTaskMessage<ChapterDownloadTask>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.get_task(msg.into())
    }
}
