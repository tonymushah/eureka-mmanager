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
    have_been_read: bool,
}

impl Actor for MangaDownloadTask {
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

impl MangaDownloadTask {
    pub(super) fn new(id: Uuid, manager: Addr<MangaDownloadManager>) -> Self {
        let (sender, _) = channel(MangaDownloadTaskState::Pending);
        Self {
            id,
            handle: None,
            sender,
            manager,
            have_been_read: false,
        }
    }
}
