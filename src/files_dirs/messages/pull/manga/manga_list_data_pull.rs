use actix::prelude::*;

use crate::{data_pulls::manga::list::MangaListDataPull, DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Copy, Hash, Default)]
pub struct MangaListDataPullMessage;

impl Message for MangaListDataPullMessage {
    type Result = ManagerCoreResult<MangaListDataPull>;
}

impl Handler<MangaListDataPullMessage> for DirsOptions {
    type Result = ManagerCoreResult<MangaListDataPull>;
    fn handle(&mut self, _msg: MangaListDataPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        MangaListDataPull::new(self.mangas.clone())
    }
}
