#[cfg(feature = "stream")]
use std::task::Poll;
use std::{
    fs::{read_dir, DirEntry, File, ReadDir},
    io::BufReader,
    iter::Flatten,
    path::PathBuf,
};

use mangadex_api_schema_rust::v5::{ChapterData, ChapterObject};
#[cfg(feature = "stream")]
use tokio_stream::Stream;

use crate::ManagerCoreResult;

#[derive(Debug)]
pub struct ChapterListDataPull {
    read_dir: Flatten<ReadDir>,
}

impl ChapterListDataPull {
    pub(crate) fn new(chapter_path: PathBuf) -> ManagerCoreResult<Self> {
        let read_dir = read_dir(chapter_path)?.flatten();
        Ok(Self { read_dir })
    }
    // TODO add cbor support
    fn dir_entry_to_chapter(entry: DirEntry) -> ManagerCoreResult<ChapterObject> {
        if !entry.path().exists()
            || !entry.path().is_dir() && !entry.path().join("data.json").exists()
        {
            return Err(crate::Error::InvalidFileName(entry.path()));
        }
        let file = BufReader::new(File::open(entry.path().join("data.json"))?);
        let o: ChapterData = serde_json::from_reader(file)?;
        Ok(o.data)
    }
}

impl Iterator for ChapterListDataPull {
    type Item = ManagerCoreResult<ChapterObject>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.read_dir.next()?;
        Some(Self::dir_entry_to_chapter(next))
    }
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl Stream for ChapterListDataPull {
    type Item = ChapterObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(entry) = self.next() {
            if let Ok(res) = entry {
                Poll::Ready(Some(res))
            } else {
                cx.waker().wake_by_ref();
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
}
