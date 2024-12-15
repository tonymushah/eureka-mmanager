pub mod messages;

use std::ops::Deref;

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
    FetchingAtHomeData,
}

pub type ChapterDownloadTaskState = DownloadTaskState<ChapterObject, ChapterDownloadingState>;

#[derive(Debug, Clone, Copy)]
pub enum DownloadMode {
    Normal,
    DataSaver,
}

impl Message for DownloadMode {
    type Result = ();
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

impl From<DownloadMode> for api_core::data_push::chapter::image::Mode {
    fn from(value: DownloadMode) -> Self {
        match value {
            DownloadMode::Normal => api_core::data_push::chapter::image::Mode::Data,
            DownloadMode::DataSaver => api_core::data_push::chapter::image::Mode::DataSaver,
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
    manager: Addr<ChapterDownloadManager>,
}

impl Actor for ChapterDownloadTask {
    type Context = Context<Self>;
    fn stopping(&mut self, ctx: &mut Self::Context) -> Running {
        if self.have_been_read
            && self.sender.is_closed()
            && std::convert::Into::<TaskState>::into(self.sender.borrow().deref()).is_finished()
        {
            self.manager
                .send(DropSingleTaskMessage(self.id))
                .into_actor(self)
                .map(|res, _, _| {
                    if let Err(er) = res {
                        log::error!("{er}");
                    }
                })
                .wait(ctx);
            Running::Stop
        } else {
            Running::Continue
        }
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
        }
    }
}

impl Handler<DownloadMode> for ChapterDownloadTask {
    type Result = <DownloadMode as Message>::Result;
    fn handle(&mut self, msg: DownloadMode, _ctx: &mut Self::Context) -> Self::Result {
        let state = std::convert::Into::<TaskState>::into(self.sender.borrow().deref());
        if !state.is_loading() {
            self.mode = msg;
        }
    }
}
