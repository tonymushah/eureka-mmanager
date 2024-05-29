use actix::prelude::*;

use crate::download::{cover::CoverDownloadManager as Manager, DownloadManager, GetManager};

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

impl GetManager<Manager> for Addr<DownloadManager> {
    async fn get(&self) -> Result<Addr<Manager>, MailboxError> {
        self.send(GetCoverDownloadManagerMessage).await
    }
}
