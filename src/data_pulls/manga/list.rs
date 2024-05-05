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
        if !entry.path().exists() || !entry.path().is_file() || !entry.path().ends_with(".json") {
            return Err(crate::Error::InvalidFileName(entry.path()));
        }
        let file = BufReader::new(File::open(entry.path())?);
        let o: MangaData = serde_json::from_reader(file)?;
        Ok(o.data)
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
                if let Ok(res) = Self::dir_entry_to_manga(entry) {
                    return Poll::Ready(Some(res));
                }
            } else {
                return Poll::Ready(None);
            }
        }
    }
}
