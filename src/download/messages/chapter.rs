use std::thread::spawn;

use actix::prelude::*;
use tokio::runtime::Runtime;

use crate::download::{
    chapter::{messages::ChapterDownloadMessage, ChapterDownloadManager as Manager},
    DownloadManager, GetManager,
};

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

pub struct DMChapterDownloadMessage(pub ChapterDownloadMessage);

impl Message for DMChapterDownloadMessage {
    type Result = crate::ManagerCoreResult<<ChapterDownloadMessage as Message>::Result>;
}

impl Handler<DMChapterDownloadMessage> for DownloadManager {
    type Result = <DMChapterDownloadMessage as Message>::Result;
    fn handle(&mut self, msg: DMChapterDownloadMessage, _ctx: &mut Self::Context) -> Self::Result {
        let chapter = self.chapter.clone();
        spawn(
            move || -> crate::ManagerCoreResult<<ChapterDownloadMessage as Message>::Result> {
                let runtime = Runtime::new()?;
                Ok(runtime.block_on(async move { chapter.send(msg.0).await })?)
            },
        )
        .join()
        .map_err(|e| crate::Error::StdThreadJoin(format!("{:#?}", e)))?
    }
}
