use std::future::Future;

use actix::prelude::*;
use dev::ToEnvelope;

use crate::{
    download::{state::DownloadManagerState, DownloadManager},
    MailBoxResult,
};

#[derive(Debug, Clone, Copy, Default)]
pub struct GetManagerStateMessage;

impl Message for GetManagerStateMessage {
    type Result = Addr<DownloadManagerState>;
}

/*
impl Handler<GetManagerStateMessage> for DownloadManagerState {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, ctx: &mut Self::Context) -> Self::Result {
        ctx.address()
    }
}
*/

impl Handler<GetManagerStateMessage> for DownloadManager {
    type Result = <GetManagerStateMessage as Message>::Result;
    fn handle(&mut self, _msg: GetManagerStateMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.state.clone()
    }
}

pub trait GetManagerState: Sync {
    fn get_manager_state(
        &self,
    ) -> impl Future<Output = MailBoxResult<Addr<DownloadManagerState>>> + Send;
}

impl<A> GetManagerState for Addr<A>
where
    A: Actor + Handler<GetManagerStateMessage>,
    <A as Actor>::Context: ToEnvelope<A, GetManagerStateMessage>,
{
    fn get_manager_state(
        &self,
    ) -> impl Future<Output = MailBoxResult<Addr<DownloadManagerState>>> + Send {
        self.send(GetManagerStateMessage)
    }
}
