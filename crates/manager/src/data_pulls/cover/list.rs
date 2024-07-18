use std::{
    fs::{read_dir, DirEntry, File, ReadDir},
    io::BufReader,
    iter::Flatten,
    path::PathBuf,
    task::Poll,
};

use mangadex_api_schema_rust::v5::{CoverData, CoverObject};
use tokio_stream::Stream;

use crate::ManagerCoreResult;

#[derive(Debug)]
pub struct CoverListDataPull {
    read_dir: Flatten<ReadDir>,
}

impl CoverListDataPull {
    pub(crate) fn new(cover_path: PathBuf) -> ManagerCoreResult<Self> {
        let read_dir = read_dir(cover_path)?.flatten();
        Ok(Self { read_dir })
    }
    fn dir_entry_to_cover(entry: DirEntry) -> ManagerCoreResult<CoverObject> {
        let path = entry.path();
        if path.exists() && path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .ok_or(crate::Error::InvalidFileName(path.clone()))?;
            if ext == "json" {
                let file = BufReader::new(File::open(&path)?);
                let o: CoverData = serde_json::from_reader(file)?;
                Ok(o.data)
            } else if ext == "cbor" {
                let file = BufReader::new(File::open(&path)?);
                let o: CoverObject = ciborium::from_reader(file)?;
                Ok(o)
            } else {
                Err(crate::Error::InvalidFileName(path.clone()))
            }
        } else {
            Err(crate::Error::InvalidFileName(path))
        }
    }
}

impl Stream for CoverListDataPull {
    type Item = CoverObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(entry) = self.read_dir.next() {
            if let Ok(res) = Self::dir_entry_to_cover(entry) {
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
