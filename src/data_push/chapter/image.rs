use std::{
    fs::File,
    io::{BufWriter, Write},
};

use bytes::Bytes;
use mangadex_api::utils::download::chapter::DownloadMode as ApiDownloadMode;
use uuid::Uuid;

use crate::{data_push::Push, download::chapter::task::DownloadMode, DirsOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Data,
    DataSaver,
}

impl From<ApiDownloadMode> for Mode {
    fn from(value: ApiDownloadMode) -> Self {
        match value {
            ApiDownloadMode::Normal => Self::Data,
            ApiDownloadMode::DataSaver => Self::DataSaver,
        }
    }
}

impl From<DownloadMode> for Mode {
    fn from(value: DownloadMode) -> Self {
        match value {
            DownloadMode::Normal => Self::Data,
            DownloadMode::DataSaver => Self::DataSaver,
        }
    }
}

impl Default for Mode {
    fn default() -> Self {
        Self::Data
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChapterImagePushEntry {
    filename: String,
    bytes: Bytes,
    id: Uuid,
    mode: Mode,
}

impl ChapterImagePushEntry {
    pub fn new(id: Uuid, filename: String, bytes: Bytes) -> Self {
        Self {
            filename,
            bytes,
            id,
            mode: Default::default(),
        }
    }
    pub fn id<I: Into<Uuid>>(self, id: I) -> Self {
        Self {
            id: id.into(),
            ..self
        }
    }
    pub fn filename<F: Into<String>>(self, filename: F) -> Self {
        Self {
            filename: filename.into(),
            ..self
        }
    }
    pub fn bytes<B: Into<Bytes>>(self, bytes: B) -> Self {
        Self {
            bytes: bytes.into(),
            ..self
        }
    }
    pub fn mode<M: Into<Mode>>(self, mode: M) -> Self {
        Self {
            mode: mode.into(),
            ..self
        }
    }
}

impl Push<ChapterImagePushEntry> for DirsOptions {
    fn push(&mut self, data: ChapterImagePushEntry) -> crate::ManagerCoreResult<()> {
        let mut file = match data.mode {
            Mode::Data => BufWriter::new(File::create(
                self.chapters_id_data_add(data.id).join(data.filename),
            )?),
            Mode::DataSaver => BufWriter::new(File::create(
                self.chapters_id_data_saver_add(data.id).join(data.filename),
            )?),
        };
        file.write_all(&data.bytes)?;
        file.flush()?;
        Ok(())
    }
}

impl Push<Vec<ChapterImagePushEntry>> for DirsOptions {
    fn push(&mut self, data: Vec<ChapterImagePushEntry>) -> crate::ManagerCoreResult<()> {
        for image in data {
            self.push(image)?;
        }
        Ok(())
    }
}
