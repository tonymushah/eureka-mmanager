use actix::prelude::*;
use uuid::Uuid;

use crate::download::{
    cover::{task::CoverDownloadTask, CoverDownloadManager},
    messages::{StartDownload, TaskStateMessage},
    state::{DownloadMessageState, TaskState},
};

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

impl Handler<CoverDownloadMessage> for CoverDownloadManager {
    type Result = <CoverDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: CoverDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        let task = self
            .tasks
            .entry(msg.id)
            .or_insert_with(|| CoverDownloadTask::new(msg.id, ctx.address()).start())
            .clone();
        let re_task = task.clone();
        self.notify.notify_waiters();

        if let DownloadMessageState::Downloading = msg.state {
            let fut = re_task
                .send(TaskStateMessage)
                .into_actor(self)
                .map_ok(move |s, _this, _ctx| {
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
