use std::{fs::File, io::BufReader, path::PathBuf, task::Poll, vec::IntoIter};

use mangadex_api_schema_rust::v5::{MangaData, MangaObject};
use tokio_stream::Stream;
use uuid::Uuid;

use actix::MessageResponse;

use crate::ManagerCoreResult;

use super::filter::IntoMangaListDataPullFilter;

#[derive(Debug, MessageResponse)]
pub struct MangaIdsListDataPull {
    manga_path: PathBuf,
    iter: IntoIter<Uuid>,
}

impl MangaIdsListDataPull {
    pub(crate) fn new(manga_path: PathBuf, ids: Vec<Uuid>) -> Self {
        Self {
            manga_path,
            iter: ids.into_iter(),
        }
    }
    fn id_to_manga(&self, entry: Uuid) -> ManagerCoreResult<MangaObject> {
        let entry = self.manga_path.join(format!("{entry}.json"));
        if entry.exists() && entry.is_file() {
            return Err(crate::Error::InvalidFileName(entry));
        }
        let file = BufReader::new(File::open(entry)?);
        let o: MangaData = serde_json::from_reader(file)?;
        Ok(o.data)
    }
}

impl IntoMangaListDataPullFilter for MangaIdsListDataPull {}

impl Stream for MangaIdsListDataPull {
    type Item = MangaObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        if let Some(entry) = self.iter.next() {
            if let Ok(res) = self.id_to_manga(entry) {
                Poll::Ready(Some(res))
            } else {
                Poll::Pending
            }
        } else {
            Poll::Ready(None)
        }
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.iter.size_hint()
    }
}
