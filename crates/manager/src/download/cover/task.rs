pub mod messages;

use actix::prelude::*;
use mangadex_api_schema_rust::v5::CoverObject;
use uuid::Uuid;

use std::{ops::Deref, sync::Arc};

use crate::{
    download::{
        messages::{DropSingleTaskMessage, TaskSubscriberMessages},
        state::{DownloadTaskState, TaskState},
    },
    recipients::Recipients,
    ArcRwLock,
};

use super::CoverDownloadManager;

#[derive(Debug, Clone, Copy)]
pub enum CoverDownloadingState {
    Preloading,
    FetchingData,
    FetchingImage,
}

pub type CoverDownloadTaskState = DownloadTaskState<CoverObject, CoverDownloadingState>;

#[derive(Debug)]
pub struct CoverDownloadTask {
    id: Uuid,
    handle: Option<SpawnHandle>,
    state: ArcRwLock<CoverDownloadTaskState>,
    subscribers: Recipients<TaskSubscriberMessages<CoverDownloadTaskState>>,
    manager: Addr<CoverDownloadManager>,
}

impl CoverDownloadTask {
    fn send_to_subscrbers(&self) -> Arc<dyn Fn(CoverDownloadTaskState) + Send + Sync + 'static> {
        let state = self.state.clone();
        let subs = self.subscribers.clone();
        Arc::new({
            move |state_to_send: CoverDownloadTaskState| {
                *state.write() = state_to_send.clone();
                subs.do_send(crate::download::messages::TaskSubscriberMessages::State(
                    state_to_send,
                ));
            }
        })
    }
}

impl Drop for CoverDownloadTask {
    fn drop(&mut self) {
        self.manager.do_send(DropSingleTaskMessage(self.id));
        self.subscribers.do_send(TaskSubscriberMessages::Dropped);
    }
}

impl Actor for CoverDownloadTask {
    type Context = Context<Self>;

    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        if std::convert::Into::<TaskState>::into(self.state.read().deref()).is_loading() {
            Running::Continue
        } else {
            Running::Stop
        }
    }
}

impl CoverDownloadTask {
    pub(super) fn new(id: Uuid, manager: Addr<CoverDownloadManager>) -> Self {
        Self {
            id,
            handle: None,
            state: Default::default(),
            manager,
            subscribers: Default::default(),
        }
    }
}
