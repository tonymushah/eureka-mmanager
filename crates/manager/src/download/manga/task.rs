pub mod messages;

use std::{ops::Deref, time::Duration};

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
    manager_handle: Option<SpawnHandle>,
}

impl Actor for MangaDownloadTask {
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

impl MangaDownloadTask {
    pub(super) fn new(id: Uuid, manager: Addr<MangaDownloadManager>) -> Self {
        let (sender, _) = channel(MangaDownloadTaskState::Pending);
        Self {
            id,
            handle: None,
            sender,
            manager,
            have_been_read: false,
            manager_handle: None,
        }
    }
}
