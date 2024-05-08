use std::{
    fs::File,
    io::{BufReader, Read},
    path::Path,
};

use actix::prelude::*;
use bytes::Bytes;
use mangadex_api_input_types::PathBuf;
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Hash, Default)]
pub struct ChapterImageDataSaverPullMessage<P: AsRef<Path>>(pub Uuid, pub P);

impl<P: AsRef<Path>> From<(Uuid, P)> for ChapterImageDataSaverPullMessage<P> {
    fn from((id, path): (Uuid, P)) -> Self {
        Self(id, path)
    }
}

impl<P: AsRef<Path>> From<ChapterImageDataSaverPullMessage<P>> for Uuid {
    fn from(value: ChapterImageDataSaverPullMessage<P>) -> Self {
        value.0
    }
}

impl<P> From<ChapterImageDataSaverPullMessage<P>> for PathBuf
where
    P: AsRef<Path>,
{
    fn from(value: ChapterImageDataSaverPullMessage<P>) -> Self {
        value.1.as_ref().to_path_buf().into()
    }
}

impl<P: AsRef<Path>> Message for ChapterImageDataSaverPullMessage<P> {
    type Result = ManagerCoreResult<Bytes>;
}

impl<P: AsRef<Path>> Handler<ChapterImageDataSaverPullMessage<P>> for DirsOptions {
    type Result = <ChapterImageDataSaverPullMessage<P> as Message>::Result;
    fn handle(
        &mut self,
        msg: ChapterImageDataSaverPullMessage<P>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let mut file = BufReader::new(File::open(
            self.chapters_id_data_saver_add(msg.0).join(msg.1),
        )?);
        let mut buf = Vec::<u8>::new();
        file.read_to_end(&mut buf)?;
        Ok(buf.into())
    }
}
