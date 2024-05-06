use std::{
    fs::{read_dir, DirEntry, File, ReadDir},
    io::BufReader,
    iter::Flatten,
    path::PathBuf,
    task::Poll,
};

use mangadex_api_schema_rust::v5::{MangaData, MangaObject};
use tokio_stream::Stream;

use crate::ManagerCoreResult;

use super::filter::IntoMangaListDataPullFilter;

#[derive(Debug)]
pub struct MangaListDataPull {
    read_dir: Flatten<ReadDir>,
}

impl MangaListDataPull {
    pub(crate) fn new(manga_path: PathBuf) -> ManagerCoreResult<Self> {
        let read_dir = read_dir(manga_path)?.flatten();
        Ok(Self { read_dir })
    }
    fn dir_entry_to_manga(entry: DirEntry) -> ManagerCoreResult<MangaObject> {
        let path = entry.path();
        if path.exists() && path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .ok_or(crate::Error::InvalidFileName(path.clone()))?;
            if ext == "json" {
                let file = BufReader::new(File::open(&path)?);
                let o: MangaData = serde_json::from_reader(file)?;
                Ok(o.data)
            } else if ext == "cbor" {
                let file = BufReader::new(File::open(&path)?);
                let o: MangaObject = ciborium::from_reader(file)?;
                Ok(o)
            } else {
                Err(crate::Error::InvalidFileName(path.clone()))
            }
        } else {
            Err(crate::Error::InvalidFileName(path))
        }
    }
}

impl IntoMangaListDataPullFilter for MangaListDataPull {}

impl Stream for MangaListDataPull {
    type Item = MangaObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(entry) = self.read_dir.next() {
                match Self::dir_entry_to_manga(entry) {
                    Ok(o) => return Poll::Ready(Some(o)),
                    Err(e) => log::error!("{}", e),
                }
            } else {
                return Poll::Ready(None);
            }
        }
    }
}
