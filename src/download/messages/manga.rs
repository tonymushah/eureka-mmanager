use actix::prelude::*;

use crate::download::{manga::MangaDownloadManager, DownloadManager};

pub struct GetMangaDownloadManagerMessage;

impl Message for GetMangaDownloadManagerMessage {
    type Result = Addr<MangaDownloadManager>;
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
