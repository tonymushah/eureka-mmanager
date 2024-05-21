use actix::{dev::MessageResponse, Actor, Message};
use mangadex_api::MangaDexClient;

use crate::download::DownloadManager;

use super::DownloadManagerState;

pub mod get;
pub mod update;

impl<M> MessageResponse<DownloadManagerState, M> for MangaDexClient
where
    M: Message<Result = Self>,
{
    fn handle(
        self,
        _ctx: &mut <DownloadManagerState as Actor>::Context,
        tx: Option<actix::prelude::dev::OneshotSender<M::Result>>,
    ) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}

impl<M> MessageResponse<DownloadManager, M> for MangaDexClient
where
    M: Message<Result = Self>,
{
    fn handle(
        self,
        _ctx: &mut <DownloadManager as Actor>::Context,
        tx: Option<actix::prelude::dev::OneshotSender<M::Result>>,
    ) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}
