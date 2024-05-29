use std::thread::spawn;

use actix::prelude::*;
use tokio::runtime::Runtime;

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
    fn handle(&mut self, msg: DMMangaDownloadMessage, _ctx: &mut Self::Context) -> Self::Result {
        let manga = self.manga.clone();
        spawn(
            move || -> crate::ManagerCoreResult<<MangaDownloadMessage as Message>::Result> {
                let runtime = Runtime::new()?;
                Ok(runtime.block_on(async move { manga.send(msg.0).await })?)
            },
        )
        .join()
        .map_err(|e| crate::Error::StdThreadJoin(format!("{:#?}", e)))?
    }
}
