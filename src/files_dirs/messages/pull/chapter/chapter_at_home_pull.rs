use actix::prelude::*;
use uuid::Uuid;

use crate::{
    data_pulls::{chapter::images::ChapterImagesData, Pull},
    DirsOptions, ManagerCoreResult,
};

#[derive(Debug, Clone, Hash, Default, Copy)]
pub struct ChapterImagesPullMessage(pub Uuid);

impl From<Uuid> for ChapterImagesPullMessage {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<ChapterImagesPullMessage> for Uuid {
    fn from(value: ChapterImagesPullMessage) -> Self {
        value.0
    }
}

impl Message for ChapterImagesPullMessage {
    type Result = ManagerCoreResult<ChapterImagesData>;
}

impl Pull<ChapterImagesData, ChapterImagesPullMessage> for DirsOptions {
    fn pull(&self, id: ChapterImagesPullMessage) -> ManagerCoreResult<ChapterImagesData> {
        self.pull(Into::<Uuid>::into(id))
    }
}
