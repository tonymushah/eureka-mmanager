use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;

use crate::download::{
    manga::task::{MangaDonwloadingState, MangaDownloadTask},
    messages::WaitForFinishedMessage,
    state::WaitForFinished,
    traits::task::CanBeWaited,
};

pub type WaitForFinishedMangaMessage = WaitForFinishedMessage<MangaObject, MangaDonwloadingState>;

impl Handler<WaitForFinishedMangaMessage> for MangaDownloadTask {
    type Result = <WaitForFinishedMangaMessage as Message>::Result;
    fn handle(
        &mut self,
        _msg: WaitForFinishedMangaMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.wait()
    }
}

impl CanBeWaited for MangaDownloadTask {
    type Ok = MangaObject;
    type Loading = MangaDonwloadingState;
    fn wait(&mut self) -> WaitForFinished<Self::Ok, Self::Loading> {
        WaitForFinished::new(self.sender.subscribe())
    }
}
