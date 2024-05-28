use std::sync::{Arc, Mutex};

use actix::prelude::*;

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
    fn handle(&mut self, msg: DMCoverDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        let mutex = Arc::new(Mutex::<
            Option<crate::ManagerCoreResult<<CoverDownloadMessage as Message>::Result>>,
        >::new(None));
        let mutex_ = mutex.clone();
        self.cover
            .send(msg.0)
            .into_actor(self)
            .map(move |d, _, _| {
                if let Ok(mut write) = mutex_.lock() {
                    write.replace(d.map_err(crate::Error::MailBox));
                }
            })
            .wait(ctx);
        let lock = mutex.lock();
        if let Ok(mut d) = lock {
            d.take().ok_or(crate::Error::NotInitialized)?
        } else {
            Err(crate::Error::NotInitialized)
        }
    }
}
