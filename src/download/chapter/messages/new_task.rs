use actix::prelude::*;
use uuid::Uuid;

use crate::download::{
    chapter::{
        task::{ChapterDownloadTask as Task, DownloadMode},
        ChapterDownloadManager as Manager,
    },
    messages::{StartDownload, TaskStateMessage},
    state::{DownloadMessageState, TaskState},
};

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
    type Result = Addr<Task>;
}

impl Handler<ChapterDownloadMessage> for Manager {
    type Result = <ChapterDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: ChapterDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        println!("new task d");
        let task = self
            .tasks
            .entry(msg.id)
            .or_insert_with(|| Task::new(msg.id, msg.mode, ctx.address()).start())
            .clone();
        let re_task = task.clone();
        println!("new task");
        self.notify.notify_waiters();

        if let DownloadMessageState::Downloading = msg.state {
            let fut = re_task
                .send(TaskStateMessage)
                .into_actor(self)
                .map_ok(move |s, _this, _ctx| {
                    if s != TaskState::Loading {
                        re_task.do_send(msg.mode);
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
