use std::future::Future;

use crate::{
    download::{chapter::ChapterDownloadManager as Manager, DownloadManager, GetManager},
    MailBoxResult,
};
use actix::prelude::*;
use dev::ToEnvelope;

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

impl<A> GetManager<Manager> for Addr<A>
where
    A: Actor + Handler<GetChapterDownloadManagerMessage>,
    <A as Actor>::Context: ToEnvelope<A, GetChapterDownloadManagerMessage>,
{
    async fn get(&self) -> Result<Addr<Manager>, MailboxError> {
        self.send(GetChapterDownloadManagerMessage).await
    }
}

pub trait GetChapterDownloadManager: Sync {
    fn get_chapter_manager(&self) -> impl Future<Output = MailBoxResult<Addr<Manager>>> + Send;
}

impl<A> GetChapterDownloadManager for A
where
    A: GetManager<Manager> + Sync,
{
    fn get_chapter_manager(&self) -> impl Future<Output = MailBoxResult<Addr<Manager>>> + Send {
        self.get()
    }
}
