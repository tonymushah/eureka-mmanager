pub mod messages;

use std::ops::Deref;

use actix::prelude::*;
use mangadex_api::utils::download::chapter::DownloadMode as Mode;
use mangadex_api_schema_rust::v5::ChapterObject;
use uuid::Uuid;

use crate::{
    download::{
        messages::{DropSingleTaskMessage, TaskSubscriberMessages},
        state::{DownloadTaskState, TaskState},
    },
    recipients::Recipients,
    ArcRwLock,
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
    state: ArcRwLock<ChapterDownloadTaskState>,
    manager: Addr<ChapterDownloadManager>,
    subscribers: Recipients<TaskSubscriberMessages<ChapterDownloadTaskState>>,
}

impl ChapterDownloadTask {
    fn sync_state_subscribers(&self) {
        self.subscribers.do_send(TaskSubscriberMessages::State(
            self.state.read().deref().clone(),
        ));
    }
}

impl Drop for ChapterDownloadTask {
    fn drop(&mut self) {
        self.manager.do_send(DropSingleTaskMessage(self.id));
        self.subscribers.do_send(TaskSubscriberMessages::Dropped);
    }
}

impl Actor for ChapterDownloadTask {
    type Context = Context<Self>;
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if std::convert::Into::<TaskState>::into(self.state.read().deref()).is_loading() {
            Running::Continue
        } else {
            Running::Stop
        }
    }
}

impl ChapterDownloadTask {
    pub(super) fn new<M: Into<DownloadMode>>(
        id: Uuid,
        mode: M,
        manager: Addr<ChapterDownloadManager>,
    ) -> Self {
        Self {
            id,
            mode: mode.into(),
            handle: None,
            state: Default::default(),
            subscribers: Default::default(),
            manager,
        }
    }
}

impl Handler<DownloadMode> for ChapterDownloadTask {
    type Result = <DownloadMode as Message>::Result;
    fn handle(&mut self, msg: DownloadMode, _ctx: &mut Self::Context) -> Self::Result {
        let state = std::convert::Into::<TaskState>::into(self.state.read().deref());
        if !state.is_loading() {
            self.mode = msg;
        }
    }
}
