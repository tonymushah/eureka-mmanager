use actix::prelude::*;
use mangadex_api_schema_rust::v5::MangaObject;

use crate::download::{
    manga::task::{MangaDonwloadingState, MangaDownloadTask},
    messages::WaitForFinishedMessage,
    state::{make_wait_for_finish_couple, WaitForFinished},
    traits::task::{CanBeWaited, Subscribe},
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
        let (rc, wait) = make_wait_for_finish_couple::<Self::Ok, Self::Loading>();
        self.subscribe(rc.into());
        wait
    }
}
