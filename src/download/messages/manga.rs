use std::future::Future;

use actix::prelude::*;
use dev::ToEnvelope;

use crate::{
    download::{manga::MangaDownloadManager as Manager, DownloadManager, GetManager},
    MailBoxResult,
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

pub trait GetMangaDownloadManager: Sync {
    fn get_manga_manager(&self) -> impl Future<Output = MailBoxResult<Addr<Manager>>> + Send;
}

impl<A> GetMangaDownloadManager for Addr<A>
where
    A: Actor + Handler<GetMangaDownloadManagerMessage>,
    <A as Actor>::Context: ToEnvelope<A, GetMangaDownloadManagerMessage>,
{
    fn get_manga_manager(&self) -> impl Future<Output = MailBoxResult<Addr<Manager>>> + Send {
        self.send(GetMangaDownloadManagerMessage)
    }
}
