pub mod messages;
pub mod task;

use std::{collections::HashMap, sync::Arc, time::Duration};

use actix::{prelude::*, WeakAddr};
use shrink_fit_wrapper::ShrinkFitWrapper;
use tokio::sync::Notify;
use uuid::Uuid;

use crate::{download::messages::StopTask, prelude::AsyncState};

use self::task::CoverDownloadTask;

use super::{
    messages::{DropSingleTaskMessage, GetTaskMessage, StartDownload},
    state::{DownloadManagerState, DownloadMessageState},
    traits::managers::TaskManager,
};

#[derive(Debug)]
pub struct CoverDownloadManager {
    state: Addr<DownloadManagerState>,
    tasks: ShrinkFitWrapper<HashMap<Uuid, WeakAddr<CoverDownloadTask>>>,
    notify: Arc<Notify>,
}

impl CoverDownloadManager {
    pub fn new(state: Addr<DownloadManagerState>) -> Self {
        Self {
            state,
            tasks: ShrinkFitWrapper::new(HashMap::new())
                .set_shrink_duration_cycle(Duration::from_secs(10 * 60)),
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
            match self.tasks.as_mut().entry(msg.id) {
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
        if let Some(task) = self.tasks.get(&id) {
            if task.upgrade().is_none() {
                self.tasks.as_mut().remove(&id);
            }
        }
        self.notify.notify_waiters();
    }
    fn get_task(&self, id: Uuid) -> Option<Addr<Self::Task>> {
        self.tasks.get(&id).and_then(WeakAddr::upgrade)
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

impl Handler<GetTaskMessage<CoverDownloadTask>> for CoverDownloadManager {
    type Result = <GetTaskMessage<CoverDownloadTask> as Message>::Result;
    fn handle(
        &mut self,
        msg: GetTaskMessage<CoverDownloadTask>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.get_task(msg.into())
    }
}

impl Drop for CoverDownloadManager {
    fn drop(&mut self) {
        self.tasks
            .values()
            .flat_map(|maybe_task| maybe_task.upgrade())
            .for_each(|task| task.do_send(StopTask));
    }
}
