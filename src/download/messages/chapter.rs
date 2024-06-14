use crate::download::{chapter::ChapterDownloadManager as Manager, DownloadManager, GetManager};
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
