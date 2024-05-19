use actix::prelude::*;

use crate::download::{
    manga::task::{MangaDownloadTask, MangaDownloadTaskState},
    messages::SubcribeMessage,
};

impl Handler<SubcribeMessage<MangaDownloadTaskState>> for MangaDownloadTask {
    type Result = <SubcribeMessage<MangaDownloadTaskState> as Message>::Result;
    fn handle(
        &mut self,
        _msg: SubcribeMessage<MangaDownloadTaskState>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        if !self.have_been_read {
            self.have_been_read = true;
        }
        Ok(self.sender.subscribe())
    }
}
