pub mod messages;

use std::{ops::Deref, sync::Arc};

use actix::prelude::*;
use futures_util::FutureExt;
use mangadex_api_schema_rust::v5::MangaObject;
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
    state: ArcRwLock<MangaDownloadTaskState>,
    subscribers: Recipients<TaskSubscriberMessages<MangaDownloadTaskState>>,
    manager: Addr<MangaDownloadManager>,
    should_stop: bool,
}

impl Drop for MangaDownloadTask {
    fn drop(&mut self) {
        if !self.should_stop {
            self.manager.do_send(DropSingleTaskMessage(self.id));
        }
        self.subscribers.do_send(TaskSubscriberMessages::Dropped);
    }
}

impl Actor for MangaDownloadTask {
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

impl Handler<StopTask> for MangaDownloadTask {
    type Result = ();
    fn handle(&mut self, _msg: StopTask, ctx: &mut Self::Context) -> Self::Result {
        self.should_stop = true;
        ctx.terminate();
    }
}

impl MangaDownloadTask {
    fn send_to_subscrbers(&self) -> Arc<dyn Fn(MangaDownloadTaskState) + Send + Sync + 'static> {
        let state = self.state.clone();
        let subs = self.subscribers.clone();
        Arc::new({
            move |state_to_send: MangaDownloadTaskState| {
                *state.write() = state_to_send.clone();
                subs.do_send(crate::download::messages::TaskSubscriberMessages::State(
                    state_to_send,
                ));
            }
        })
    }
}

impl MangaDownloadTask {
    pub(super) fn new(id: Uuid, manager: Addr<MangaDownloadManager>) -> Self {
        Self {
            id,
            handle: None,
            state: Default::default(),
            manager,
            subscribers: Default::default(),
            should_stop: false,
        }
    }
}

impl Handler<FilesDirSubscriberMessage> for MangaDownloadTask {
    type Result = ();
    fn handle(&mut self, msg: FilesDirSubscriberMessage, ctx: &mut Self::Context) -> Self::Result {
        if let FilesDirSubscriberMessage::RemovedManga { id } = msg
            && id == self.id
        {
            if self.state().is_finished() {
                *self.state.write() = MangaDownloadTaskState::Pending;
            } else if self.state().is_loading() {
                self.cancel(ctx);
            }
        }
    }
}
