use std::{
    fs::File,
    io::{self, BufWriter, Read, Write},
};

use uuid::Uuid;

use crate::{data_push::Push, DirsOptions};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Mode {
    Data,
    DataSaver,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Data
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ChapterImagePushEntry<R> {
    filename: String,
    reader: R,
    id: Uuid,
    mode: Mode,
}

impl<R> ChapterImagePushEntry<R> {
    pub fn new(id: Uuid, filename: String, reader: R) -> Self {
        Self {
            filename,
            reader,
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
    pub fn reader(self, reader: R) -> Self {
        Self { reader, ..self }
    }
    pub fn mode<M: Into<Mode>>(self, mode: M) -> Self {
        Self {
            mode: mode.into(),
            ..self
        }
    }
}

impl<R> Push<ChapterImagePushEntry<R>> for DirsOptions
where
    R: Read,
{
    type Error = crate::Error;
    fn push(&mut self, mut data: ChapterImagePushEntry<R>) -> crate::ManagerCoreResult<()> {
        let mut file = match data.mode {
            Mode::Data => BufWriter::new(File::create(
                self.chapters_id_data_add(data.id).join(data.filename),
            )?),
            Mode::DataSaver => BufWriter::new(File::create(
                self.chapters_id_data_saver_add(data.id).join(data.filename),
            )?),
        };
        {
            io::copy(&mut data.reader, &mut file)?;
        }
        file.flush()?;
        Ok(())
    }
}

impl<R> Push<Vec<ChapterImagePushEntry<R>>> for DirsOptions
where
    R: Read,
{
    type Error = crate::Error;
    fn push(&mut self, data: Vec<ChapterImagePushEntry<R>>) -> crate::ManagerCoreResult<()> {
        for image in data {
            self.push(image)?;
        }
        Ok(())
    }
}
