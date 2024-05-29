use crate::download::{chapter::ChapterDownloadManager as Manager, DownloadManager, GetManager};
use actix::prelude::*;

pub struct GetChapterDownloadManagerMessage;

impl Message for GetChapterDownloadManagerMessage {
    type Result = Addr<Manager>;
}

impl Handler<GetChapterDownloadManagerMessage> for DownloadManager {
    type Result = <GetChapterDownloadManagerMessage as Message>::Result;
    fn handle(
        &mut self,
        _msg: GetChapterDownloadManagerMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.chapter.clone()
    }
}

impl GetManager<Manager> for Addr<DownloadManager> {
    async fn get(&self) -> Result<Addr<Manager>, MailboxError> {
        self.send(GetChapterDownloadManagerMessage).await
    }
}
