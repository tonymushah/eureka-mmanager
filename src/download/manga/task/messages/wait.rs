use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;

use crate::download::{
    manga::task::{MangaDonwloadingState, MangaDownloadTask},
    messages::WaitForFinishedMessage,
    state::WaitForFinished,
};

impl Handler<WaitForFinishedMessage<MangaObject, MangaDonwloadingState>> for MangaDownloadTask {
    type Result = <WaitForFinishedMessage<MangaObject, MangaDonwloadingState> as Message>::Result;
    fn handle(
        &mut self,
        _msg: WaitForFinishedMessage<MangaObject, MangaDonwloadingState>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if !self.have_been_read {
            self.have_been_read = true;
        }
        WaitForFinished::new(self.sender.subscribe())
    }
}
