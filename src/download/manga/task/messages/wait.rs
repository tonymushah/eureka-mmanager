use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;

use crate::download::{
    manga::task::{MangaDonwloadingState, MangaDownloadTask},
    messages::WaitForFinishedMessage,
    state::WaitForFinished,
};

pub type WaitForFinishedMangaMessage = WaitForFinishedMessage<MangaObject, MangaDonwloadingState>;

impl Handler<WaitForFinishedMangaMessage> for MangaDownloadTask {
    type Result = <WaitForFinishedMangaMessage as Message>::Result;
    fn handle(
        &mut self,
        _msg: WaitForFinishedMangaMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if !self.have_been_read {
            self.have_been_read = true;
        }
        WaitForFinished::new(self.sender.subscribe())
    }
}
