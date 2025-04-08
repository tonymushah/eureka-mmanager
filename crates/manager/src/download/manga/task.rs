pub mod messages;

use std::ops::Deref;

use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;
use tokio::sync::watch::{channel, Sender};
use uuid::Uuid;

use crate::download::{
    messages::DropSingleTaskMessage,
    state::{DownloadTaskState, TaskState},
};

use super::MangaDownloadManager;

#[derive(Debug, Clone, Copy)]
pub enum MangaDonwloadingState {
    Preloading,
    FetchingData,
}

pub type MangaDownloadTaskState = DownloadTaskState<MangaObject, MangaDonwloadingState>;

#[derive(Debug, MessageResponse)]
pub struct MangaDownloadTask {
    id: Uuid,
    handle: Option<SpawnHandle>,
    sender: Sender<MangaDownloadTaskState>,
    manager: Addr<MangaDownloadManager>,
}

impl Drop for MangaDownloadTask {
    fn drop(&mut self) {
        self.manager.do_send(DropSingleTaskMessage(self.id));
    }
}

impl Actor for MangaDownloadTask {
    type Context = Context<Self>;
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if std::convert::Into::<TaskState>::into(self.sender.borrow().deref()).is_loading() {
            Running::Continue
        } else {
            Running::Stop
        }
    }
}

impl MangaDownloadTask {
    pub(super) fn new(id: Uuid, manager: Addr<MangaDownloadManager>) -> Self {
        let (sender, _) = channel(MangaDownloadTaskState::Pending);
        Self {
            id,
            handle: None,
            sender,
            manager,
        }
    }
}
