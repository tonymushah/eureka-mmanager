use std::{sync::Arc, sync::Mutex};

use actix::prelude::*;

use crate::download::{
    manga::{messages::MangaDownloadMessage, MangaDownloadManager as Manager},
    DownloadManager, GetManager,
};

pub struct GetMangaDownloadManagerMessage;

impl Message for GetMangaDownloadManagerMessage {
    type Result = Addr<Manager>;
}

impl Handler<GetMangaDownloadManagerMessage> for DownloadManager {
    type Result = <GetMangaDownloadManagerMessage as Message>::Result;
    fn handle(
        &mut self,
        _msg: GetMangaDownloadManagerMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.manga.clone()
    }
}

impl GetManager<Manager> for Addr<DownloadManager> {
    async fn get(&self) -> Result<Addr<Manager>, MailboxError> {
        self.send(GetMangaDownloadManagerMessage).await
    }
}

pub struct DMMangaDownloadMessage(pub MangaDownloadMessage);

impl Message for DMMangaDownloadMessage {
    type Result = crate::ManagerCoreResult<<MangaDownloadMessage as Message>::Result>;
}

impl Handler<DMMangaDownloadMessage> for DownloadManager {
    type Result = <DMMangaDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: DMMangaDownloadMessage, ctx: &mut Self::Context) -> Self::Result {
        let mutex = Arc::new(Mutex::<
            Option<crate::ManagerCoreResult<<MangaDownloadMessage as Message>::Result>>,
        >::new(None));
        let mutex_ = mutex.clone();
        self.manga
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
