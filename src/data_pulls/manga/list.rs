use std::{
    collections::HashMap,
    fs::{read_dir, DirEntry, File, ReadDir},
    io::BufReader,
    iter::Flatten,
    path::PathBuf,
    task::Poll,
};

use mangadex_api_schema_rust::v5::{MangaData, MangaObject};
use mangadex_api_types_rust::Language;
use tokio_stream::Stream;
use uuid::Uuid;

use crate::ManagerCoreResult;

#[derive(Debug)]
pub struct MangaListDataPull {
    available_langs: HashMap<Uuid, Vec<Language>>,
    read_dir: Flatten<ReadDir>,
}

impl MangaListDataPull {
    pub(crate) fn new(manga_path: PathBuf) -> ManagerCoreResult<Self> {
        let read_dir = read_dir(manga_path)?.flatten();
        Ok(Self {
            read_dir,
            available_langs: HashMap::default(),
        })
    }
    pub(crate) fn with_available_langs(
        mut self,
        available_langs: HashMap<Uuid, Vec<Language>>,
    ) -> Self {
        self.available_langs = available_langs;
        self
    }
    fn dir_entry_to_manga(&self, entry: DirEntry) -> ManagerCoreResult<MangaObject> {
        let path = entry.path();
        if path.exists() && path.is_file() {
            let ext = path
                .extension()
                .and_then(|e| e.to_str())
                .ok_or(crate::Error::InvalidFileName(path.clone()))?;
            let mut data = if ext == "json" {
                let file = BufReader::new(File::open(&path)?);
                let o: MangaData = serde_json::from_reader(file)?;
                o.data
            } else if ext == "cbor" {
                let file = BufReader::new(File::open(&path)?);
                let o: MangaObject = ciborium::from_reader(file)?;
                o
            } else {
                return Err(crate::Error::InvalidFileName(path.clone()));
            };
            if let Some(langs) = self.available_langs.get(&data.id) {
                data.attributes
                    .available_translated_languages
                    .clone_from(langs);
            }
            Ok(data)
        } else {
            Err(crate::Error::InvalidFileName(path))
        }
    }
}

impl Stream for MangaListDataPull {
    type Item = MangaObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(entry) = self.read_dir.next() {
            match self.dir_entry_to_manga(entry) {
                Ok(o) => Poll::Ready(Some(o)),
                Err(e) => {
                    log::error!("{}", e);
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        } else {
            Poll::Ready(None)
        }
    }
}
