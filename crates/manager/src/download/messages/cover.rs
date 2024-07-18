use std::future::Future;

use actix::prelude::*;
use dev::ToEnvelope;

use crate::{
    download::{cover::CoverDownloadManager as Manager, DownloadManager, GetManager},
    MailBoxResult,
};

pub struct GetCoverDownloadManagerMessage;

impl Message for GetCoverDownloadManagerMessage {
    type Result = Addr<Manager>;
}

impl Handler<GetCoverDownloadManagerMessage> for DownloadManager {
    type Result = <GetCoverDownloadManagerMessage as Message>::Result;
    fn handle(
        &mut self,
        _msg: GetCoverDownloadManagerMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.cover.clone()
    }
}

impl<A> GetManager<Manager> for Addr<A>
where
    A: Actor + Handler<GetCoverDownloadManagerMessage>,
    <A as Actor>::Context: ToEnvelope<A, GetCoverDownloadManagerMessage>,
{
    async fn get(&self) -> Result<Addr<Manager>, MailboxError> {
        self.send(GetCoverDownloadManagerMessage).await
    }
}

pub trait GetCoverDownloadManager: Sync {
    fn get_cover_manager(&self) -> impl Future<Output = MailBoxResult<Addr<Manager>>> + Send;
}

impl<A> GetCoverDownloadManager for A
where
    A: GetManager<Manager> + Sync,
{
    fn get_cover_manager(&self) -> impl Future<Output = MailBoxResult<Addr<Manager>>> + Send {
        self.get()
    }
}
