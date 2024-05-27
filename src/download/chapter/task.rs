use std::{ops::Deref, time::Duration};

use actix::prelude::*;
use mangadex_api::utils::download::chapter::DownloadMode as Mode;
use mangadex_api_schema_rust::v5::ChapterObject;
use tokio::sync::watch::{channel, Sender};
use uuid::Uuid;

use crate::download::{
    messages::DropSingleTaskMessage,
    state::{DownloadTaskState, TaskState},
};

use super::ChapterDownloadManager;

#[derive(Debug, Clone)]
pub enum ChapterDownloadingState {
    Preloading,
    FetchingData,
    FetchingImage {
        filename: String,
        index: usize,
        len: usize,
    },
}

pub type ChapterDownloadTaskState = DownloadTaskState<ChapterObject, ChapterDownloadingState>;

#[derive(Debug, Clone, Copy)]
pub enum DownloadMode {
    Normal,
    DataSaver,
}

impl From<DownloadMode> for Mode {
    fn from(value: DownloadMode) -> Self {
        match value {
            DownloadMode::Normal => Self::Normal,
            DownloadMode::DataSaver => Self::DataSaver,
        }
    }
}

impl From<Mode> for DownloadMode {
    fn from(value: Mode) -> Self {
        match value {
            Mode::Normal => Self::Normal,
            Mode::DataSaver => Self::DataSaver,
        }
    }
}

#[derive(Debug)]
pub struct ChapterDownloadTask {
    id: Uuid,
    mode: DownloadMode,
    handle: Option<SpawnHandle>,
    sender: Sender<ChapterDownloadTaskState>,
    have_been_read: bool,
    manager_handle: Option<SpawnHandle>,
    manager: Addr<ChapterDownloadManager>,
}

impl Actor for ChapterDownloadTask {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        self.manager_handle = Some(ctx.run_interval(Duration::from_millis(500), |this, _ctx| {
            if this.have_been_read
                && this.sender.is_closed()
                && std::convert::Into::<TaskState>::into(this.sender.borrow().deref()).is_finished()
            {
                this.manager.do_send(DropSingleTaskMessage(this.id));
            }
        }));
    }
}

impl ChapterDownloadTask {
    pub(super) fn new<M: Into<DownloadMode>>(
        id: Uuid,
        mode: M,
        manager: Addr<ChapterDownloadManager>,
    ) -> Self {
        let (sender, _) = channel(ChapterDownloadTaskState::Pending);
        Self {
            id,
            mode: mode.into(),
            handle: None,
            sender,
            manager,
            have_been_read: false,
            manager_handle: None,
        }
    }
}
