use std::{
    fs::{read_dir, DirEntry, File},
    io::BufReader,
    path::PathBuf,
    task::Poll,
};

use mangadex_api_schema_rust::v5::{ChapterData, ChapterObject};
use tokio_stream::Stream;

use crate::ManagerCoreResult;

use std::vec::IntoIter;

#[derive(Debug)]
pub struct ChapterListDataPull {
    read_dir: IntoIter<DirEntry>,
}

impl ChapterListDataPull {
    pub(crate) fn new(chapter_path: PathBuf) -> ManagerCoreResult<Self> {
        let read_dir = read_dir(chapter_path)?
            .flatten()
            .collect::<Vec<DirEntry>>()
            .into_iter();
        Ok(Self { read_dir })
    }
    fn dir_entry_to_chapter(entry: DirEntry) -> ManagerCoreResult<ChapterObject> {
        if entry.path().exists() && entry.path().is_dir() && entry.path().join("data.json").exists()
        {
            return Err(crate::Error::InvalidFileName(entry.path()));
        }
        let file = BufReader::new(File::open(entry.path().join("data.json"))?);
        let o: ChapterData = serde_json::from_reader(file)?;
        Ok(o.data)
    }
}

impl Stream for ChapterListDataPull {
    type Item = ChapterObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(entry) = self.read_dir.next() {
            if let Ok(res) = Self::dir_entry_to_chapter(entry) {
                Poll::Ready(Some(res))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.read_dir.size_hint()
    }
}
