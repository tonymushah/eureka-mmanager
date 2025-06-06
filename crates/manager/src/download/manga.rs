pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc};

use actix::{prelude::*, WeakAddr};
use tokio::sync::Notify;
use uuid::Uuid;

use self::task::MangaDownloadTask;

use super::{
    messages::{DropSingleTaskMessage, GetTaskMessage, StartDownload},
    state::{DownloadManagerState, DownloadMessageState},
    traits::{managers::TaskManager, task::AsyncState},
};

#[derive(Debug)]
pub struct MangaDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: HashMap<Uuid, WeakAddr<MangaDownloadTask>>,
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
        if let Some(task) = self.tasks.get(&id) {
            if task.upgrade().is_none() {
                self.tasks.remove(&id);
            }
        }
        self.notify.notify_waiters();
    }
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
                        let tsk = Self::Task::new(msg.id, ctx.address()).start();
                        let _weak = std::mem::replace(weak, tsk.downgrade());
                        tsk
                    }
                }
                std::collections::hash_map::Entry::Vacant(vacant_entry) => {
                    let tsk = Self::Task::new(msg.id, ctx.address()).start();
                    vacant_entry.insert(tsk.downgrade());
                    tsk
                }
            }
        };
        let re_task = task.clone();
        self.notify.notify_waiters();

        if let DownloadMessageState::Downloading = msg.state {
            let fut = async move {
                let s = re_task.state().await?;
                if !s.is_loading() {
                    re_task.send(StartDownload).await?;
                }
                Ok::<_, actix::MailboxError>(())
            }
            .into_actor(self)
            .map(|s, _, _| {
                if let Err(er) = s {
                    log::error!("{er}");
                }
            });
            ctx.wait(fut)
        }
        task
    }
    fn get_task(&self, id: Uuid) -> Option<Addr<Self::Task>> {
        self.tasks.get(&id).and_then(WeakAddr::upgrade)
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

impl Handler<GetTaskMessage<MangaDownloadTask>> for MangaDownloadManager {
    type Result = <GetTaskMessage<MangaDownloadTask> as Message>::Result;
    fn handle(
        &mut self,
        msg: GetTaskMessage<MangaDownloadTask>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.get_task(msg.into())
    }
}
