#[cfg(feature = "stream")]
use std::task::Poll;
use std::{collections::HashMap, fs::File, io::BufReader, path::PathBuf, vec::IntoIter};

use mangadex_api_schema_rust::v5::{MangaData, MangaObject};
use mangadex_api_types_rust::Language;
#[cfg(feature = "stream")]
use tokio_stream::Stream;
use uuid::Uuid;

use crate::ManagerCoreResult;

#[derive(Debug)]
#[cfg_attr(feature = "actix", derive(actix::MessageResponse))]
pub struct MangaIdsListDataPull {
    available_langs: HashMap<Uuid, Vec<Language>>,
    manga_path: PathBuf,
    iter: IntoIter<Uuid>,
}

impl MangaIdsListDataPull {
    pub(crate) fn new(manga_path: PathBuf, ids: Vec<Uuid>) -> Self {
        Self {
            manga_path,
            iter: ids.into_iter(),
            available_langs: Default::default(),
        }
    }
    pub fn with_available_langs(mut self, available_langs: HashMap<Uuid, Vec<Language>>) -> Self {
        self.available_langs = available_langs;
        self
    }
    fn id_to_manga_json(&self, entry: &Uuid) -> ManagerCoreResult<MangaObject> {
        let entry = self.manga_path.join(format!("{entry}.json"));
        if !entry.exists() || !entry.is_file() {
            return Err(crate::Error::InvalidFileName(entry));
        }
        let file = BufReader::new(File::open(entry)?);
        let o: MangaData = serde_json::from_reader(file)?;
        Ok(o.data)
    }
    fn id_to_manga_cbor(&self, entry: &Uuid) -> ManagerCoreResult<MangaObject> {
        let entry = self.manga_path.join(format!("{entry}.cbor"));
        if !entry.exists() || !entry.is_file() {
            return Err(crate::Error::InvalidFileName(entry));
        }
        let file = BufReader::new(File::open(entry)?);
        let o: MangaObject = ciborium::from_reader(file)?;
        Ok(o)
    }
    fn id_to_manga(&self, entry: Uuid) -> ManagerCoreResult<MangaObject> {
        if let Ok(mut o) = self.id_to_manga_cbor(&entry) {
            if let Some(langs) = self.available_langs.get(&o.id) {
                o.attributes
                    .available_translated_languages
                    .clone_from(langs);
            }
            Ok(o)
        } else {
            self.id_to_manga_json(&entry)
        }
    }
}

impl Iterator for MangaIdsListDataPull {
    type Item = ManagerCoreResult<MangaObject>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        Some(self.id_to_manga(next))
    }
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl Stream for MangaIdsListDataPull {
    type Item = MangaObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(entry) = self.next() {
            match entry {
                Ok(d) => Poll::Ready(Some(d)),
                Err(_e) => {
                    #[cfg(feature = "log")]
                    log::error!("{_e}");
                    cx.waker().wake_by_ref();
                    Poll::Pending
                }
            }
        } else {
            Poll::Ready(None)
        }
    }
}
