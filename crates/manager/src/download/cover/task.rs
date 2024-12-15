pub mod messages;

use actix::prelude::*;
use mangadex_api_schema_rust::v5::CoverObject;
use tokio::sync::watch::{channel, Sender};
use uuid::Uuid;

use std::ops::Deref;

use crate::download::{
    messages::DropSingleTaskMessage,
    state::{DownloadTaskState, TaskState},
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
    sender: Sender<CoverDownloadTaskState>,
    manager: Addr<CoverDownloadManager>,
    have_been_read: bool,
}

impl Actor for CoverDownloadTask {
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

impl CoverDownloadTask {
    pub(super) fn new(id: Uuid, manager: Addr<CoverDownloadManager>) -> Self {
        let (sender, _) = channel(CoverDownloadTaskState::Pending);
        Self {
            id,
            handle: None,
            sender,
            manager,
            have_been_read: false,
        }
    }
}
