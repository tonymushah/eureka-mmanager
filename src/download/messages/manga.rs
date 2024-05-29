use actix::prelude::*;

use crate::download::{manga::MangaDownloadManager as Manager, DownloadManager, GetManager};

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
