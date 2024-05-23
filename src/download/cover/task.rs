use actix::prelude::*;
use mangadex_api_schema_rust::v5::CoverObject;
use tokio::sync::watch::{channel, Sender};
use uuid::Uuid;

use std::{ops::Deref, time::Duration};

use crate::download::{
    messages::DropSingleTaskMessage,
    state::{DownloadTaskState, TaskState},
};

use super::CoverDownloadManager;

#[derive(Debug)]
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
    manager_handle: Option<SpawnHandle>,
}

impl Actor for CoverDownloadTask {
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

impl CoverDownloadTask {
    pub(super) fn new(id: Uuid, manager: Addr<CoverDownloadManager>) -> Self {
        let (sender, _) = channel(CoverDownloadTaskState::Pending);
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
