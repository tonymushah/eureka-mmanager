use std::{fs::read_dir, path::Path};

use actix::prelude::*;
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Hash, Default, Copy)]
pub struct ChapterImagesPullMessage(pub Uuid);

#[derive(Debug, Clone, Hash, Default, MessageResponse)]
pub struct ChapterImagesData {
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
}

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

fn m_read_dir<P: AsRef<Path>>(path: P) -> Vec<String> {
    read_dir(path)
        .map(|dir| {
            dir.flatten()
                .filter(|f| {
                    f.path().is_file()
                        && f.path()
                            .extension()
                            .map(|ext| ext != "json")
                            .unwrap_or_default()
                })
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn string_f_usize_sort(data: &mut [String]) -> Result<(), regex::Error> {
    let regex = regex::Regex::new(r"\d+")?;
    data.sort_by(|a, b| {
        let a = regex
            .captures(a)
            .and_then(|c| c.get(0)?.as_str().parse::<usize>().ok());
        let b = regex
            .captures(b)
            .and_then(|c| c.get(0)?.as_str().parse::<usize>().ok());
        a.cmp(&b)
    });
    Ok(())
}

impl Handler<ChapterImagesPullMessage> for DirsOptions {
    type Result = <ChapterImagesPullMessage as Message>::Result;
    fn handle(&mut self, msg: ChapterImagesPullMessage, _ctx: &mut Self::Context) -> Self::Result {
        let mut data = m_read_dir(self.chapters_id_data_add(msg.into()));
        string_f_usize_sort(&mut data)?;
        let mut data_saver = m_read_dir(self.chapters_id_data_saver_add(msg.into()));
        string_f_usize_sort(&mut data_saver)?;
        Ok(ChapterImagesData { data, data_saver })
    }
}
