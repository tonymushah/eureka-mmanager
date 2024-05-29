use std::thread::spawn;

use actix::prelude::*;
use tokio::runtime::Runtime;

use crate::download::{
    cover::{messages::new_task::CoverDownloadMessage, CoverDownloadManager as Manager},
    DownloadManager, GetManager,
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

impl GetManager<Manager> for Addr<DownloadManager> {
    async fn get(&self) -> Result<Addr<Manager>, MailboxError> {
        self.send(GetCoverDownloadManagerMessage).await
    }
}

pub struct DMCoverDownloadMessage(pub CoverDownloadMessage);

impl Message for DMCoverDownloadMessage {
    type Result = crate::ManagerCoreResult<<CoverDownloadMessage as Message>::Result>;
}

impl Handler<DMCoverDownloadMessage> for DownloadManager {
    type Result = <DMCoverDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: DMCoverDownloadMessage, _ctx: &mut Self::Context) -> Self::Result {
        let cover = self.cover.clone();
        spawn(
            move || -> crate::ManagerCoreResult<<CoverDownloadMessage as Message>::Result> {
                let runtime = Runtime::new()?;
                Ok(runtime.block_on(async move { cover.send(msg.0).await })?)
            },
        )
        .join()
        .map_err(|e| crate::Error::StdThreadJoin(format!("{:#?}", e)))?
    }
}
