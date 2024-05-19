use actix::prelude::*;
use uuid::Uuid;

use crate::download::{
    manga::{task::MangaDownloadTask, MangaDownloadManager},
    messages::{StartDownload, TaskStateMessage},
    state::{DownloadMessageState, TaskState},
};

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

impl Handler<MangaDownloadMessage> for MangaDownloadManager {
    type Result = <MangaDownloadMessage as Message>::Result;
    // TODO Add support for the DownloadState
    fn handle(&mut self, msg: MangaDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        let task = self
            .tasks
            .entry(msg.id)
            .or_insert_with(|| {
                MangaDownloadTask::new(
                    self.dir_option.clone(),
                    self.client.clone(),
                    self.history.clone(),
                    msg.id,
                    ctx.address(),
                )
                .start()
            })
            .clone();
        let re_task = task.clone();

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
