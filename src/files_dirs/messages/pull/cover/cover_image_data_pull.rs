use std::{
    fs::File,
    io::{BufReader, Read},
};

use actix::prelude::*;
use bytes::Bytes;
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

use super::CoverDataPullMessage;

#[derive(Debug, Clone, Hash, Default)]
pub struct CoverImageDataPullMessage(pub Uuid);

impl From<Uuid> for CoverImageDataPullMessage {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<CoverImageDataPullMessage> for Uuid {
    fn from(value: CoverImageDataPullMessage) -> Self {
        value.0
    }
}

impl Message for CoverImageDataPullMessage {
    type Result = ManagerCoreResult<Bytes>;
}

impl Handler<CoverImageDataPullMessage> for DirsOptions {
    type Result = <CoverImageDataPullMessage as Message>::Result;
    fn handle(&mut self, msg: CoverImageDataPullMessage, ctx: &mut Self::Context) -> Self::Result {
        let filename = self
            .handle(CoverDataPullMessage(msg.0), ctx)?
            .attributes
            .file_name;
        let mut file = BufReader::new(File::open(self.cover_images_add(filename))?);
        let mut buf = Vec::<u8>::new();
        file.read_to_end(&mut buf)?;
        Ok(buf.into())
    }
}
