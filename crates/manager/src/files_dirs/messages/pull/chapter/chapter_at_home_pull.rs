use std::{fs::read_dir, ops::Deref, path::Path};

use actix::prelude::*;
use uuid::Uuid;

use crate::{
    data_pulls::{chapter::images::ChapterImagesData, Pull},
    files_dirs::FileExtension,
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
        self.deref()
            .pull(Into::<Uuid>::into(id))
            .map_err(|e: api_core::Error| e.into())
    }

    type Error = crate::Error;
}

fn get_data<P: AsRef<Path>>(p: P) -> Vec<String> {
    read_dir(p)
        .map(|read| {
            read.flatten()
                .map(|p| p.path())
                .filter(|p| p.is_image())
                .flat_map(|p| Some(String::from(p.as_os_str().to_str()?)))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

impl Handler<ChapterImagesPullMessage> for DirsOptions {
    type Result = <ChapterImagesPullMessage as Message>::Result;
    fn handle(&mut self, msg: ChapterImagesPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        let data = get_data(self.chapters_id_data_add(msg.0));
        let data_saver = get_data(self.chapters_id_data_saver_add(msg.0));
        Ok(ChapterImagesData { data, data_saver })
    }
}
