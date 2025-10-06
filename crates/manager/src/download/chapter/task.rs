pub mod messages;

use std::ops::Deref;

use actix::prelude::*;
use futures_util::FutureExt;
use mangadex_api::utils::download::chapter::DownloadMode as Mode;
use mangadex_api_schema_rust::v5::ChapterObject;
use uuid::Uuid;

use crate::{
    ArcRwLock,
    download::{
        messages::{DropSingleTaskMessage, StopTask, TaskSubscriberMessages},
        state::{DownloadTaskState, TaskState},
        traits::task::{Cancelable, State},
    },
    files_dirs::{
        events::FilesDirSubscriberMessage, messages::subscribe::DirsOptionsSubscribeMessage,
    },
    recipients::{MaybeWeakRecipient, Recipients},
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
            _ => Self::Normal,
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
    should_stop: bool,
    force_port_443: bool,
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
        if !self.should_stop {
            self.manager.do_send(DropSingleTaskMessage(self.id));
        }
        self.subscribers.do_send(TaskSubscriberMessages::Dropped);
    }
}

impl Actor for ChapterDownloadTask {
    type Context = Context<Self>;
    fn started(&mut self, ctx: &mut Self::Context) {
        let addr = ctx.address();

        let manager = self.manager.clone();
        async move {
            manager
                .send(DirsOptionsSubscribeMessage(MaybeWeakRecipient::Weak(
                    addr.downgrade().into(),
                )))
                .await
        }
        .map(|d| {
            if let Err(err) = d {
                log::error!("{err}");
            }
        })
        .into_actor(self)
        .wait(ctx);
    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if std::convert::Into::<TaskState>::into(self.state.read().deref()).is_loading()
            || self.subscribers.has_connection()
        {
            Running::Continue
        } else {
            Running::Stop
        }
    }
}

impl Handler<StopTask> for ChapterDownloadTask {
    type Result = ();
    fn handle(&mut self, _msg: StopTask, ctx: &mut Self::Context) -> Self::Result {
        self.should_stop = true;
        ctx.terminate();
    }
}

impl ChapterDownloadTask {
    pub(super) fn new<M: Into<DownloadMode>>(
        id: Uuid,
        mode: M,
        force_port_443: bool,
        manager: Addr<ChapterDownloadManager>,
    ) -> Self {
        Self {
            id,
            mode: mode.into(),
            handle: None,
            state: Default::default(),
            subscribers: Default::default(),
            manager,
            should_stop: false,
            force_port_443,
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

impl Handler<FilesDirSubscriberMessage> for ChapterDownloadTask {
    type Result = ();
    fn handle(&mut self, msg: FilesDirSubscriberMessage, ctx: &mut Self::Context) -> Self::Result {
        if let FilesDirSubscriberMessage::RemovedChapter { id } = msg
            && id == self.id
        {
            if self.state().is_finished() {
                *self.state.write() = ChapterDownloadTaskState::Pending;
            } else if self.state().is_loading() {
                self.cancel(ctx);
            }
        }
    }
}
