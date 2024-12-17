use std::{fs::File, path::Path};

use actix::prelude::*;
use mangadex_api_input_types::PathBuf;
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Hash, Default)]
pub struct ChapterImageDataPullMessage<P: AsRef<Path>>(pub Uuid, pub P);

impl<P: AsRef<Path>> From<(Uuid, P)> for ChapterImageDataPullMessage<P> {
    fn from((id, path): (Uuid, P)) -> Self {
        Self(id, path)
    }
}

impl<P: AsRef<Path>> From<ChapterImageDataPullMessage<P>> for Uuid {
    fn from(value: ChapterImageDataPullMessage<P>) -> Self {
        value.0
    }
}

impl<P> From<ChapterImageDataPullMessage<P>> for PathBuf
where
    P: AsRef<Path>,
{
    fn from(value: ChapterImageDataPullMessage<P>) -> Self {
        value.1.as_ref().to_path_buf().into()
    }
}

impl<P: AsRef<Path>> Message for ChapterImageDataPullMessage<P> {
    type Result = ManagerCoreResult<File>;
}

impl<P: AsRef<Path>> Handler<ChapterImageDataPullMessage<P>> for DirsOptions {
    type Result = <ChapterImageDataPullMessage<P> as Message>::Result;
    fn handle(
        &mut self,
        msg: ChapterImageDataPullMessage<P>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        Ok(File::open(self.chapters_id_data_add(msg.0).join(msg.1))?)
    }
}
