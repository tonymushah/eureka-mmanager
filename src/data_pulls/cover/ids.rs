use std::{fs::File, io::BufReader, path::PathBuf, task::Poll, vec::IntoIter};

use mangadex_api_schema_rust::v5::{CoverData, CoverObject};
use tokio_stream::Stream;
use uuid::Uuid;

use actix::MessageResponse;

use crate::ManagerCoreResult;

#[derive(Debug, MessageResponse)]
pub struct CoverIdsListDataPull {
    cover_path: PathBuf,
    iter: IntoIter<Uuid>,
}

impl CoverIdsListDataPull {
    pub(crate) fn new(cover_path: PathBuf, ids: Vec<Uuid>) -> Self {
        Self {
            cover_path,
            iter: ids.into_iter(),
        }
    }
    fn id_to_cover_json(&self, entry: &Uuid) -> ManagerCoreResult<CoverObject> {
        let entry = self.cover_path.join(format!("{entry}.json"));
        if !entry.exists() || !entry.is_file() {
            return Err(crate::Error::InvalidFileName(entry));
        }
        let file = BufReader::new(File::open(entry)?);
        let o: CoverData = serde_json::from_reader(file)?;
        Ok(o.data)
    }
    fn id_to_cover_cbor(&self, entry: &Uuid) -> ManagerCoreResult<CoverObject> {
        let entry = self.cover_path.join(format!("{entry}.cbor"));
        if !entry.exists() || !entry.is_file() {
            return Err(crate::Error::InvalidFileName(entry));
        }
        let file = BufReader::new(File::open(entry)?);
        let o: CoverObject = ciborium::from_reader(file)?;
        Ok(o)
    }
    fn id_to_cover(&self, entry: Uuid) -> ManagerCoreResult<CoverObject> {
        if let Ok(o) = self.id_to_cover_cbor(&entry) {
            Ok(o)
        } else {
            self.id_to_cover_json(&entry)
        }
    }
}

impl Stream for CoverIdsListDataPull {
    type Item = CoverObject;

    fn poll_next(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> Poll<Option<Self::Item>> {
        loop {
            if let Some(entry) = self.iter.next() {
                if let Ok(res) = self.id_to_cover(entry) {
                    return Poll::Ready(Some(res));
                }
            } else {
                return Poll::Ready(None);
            }
        }
    }
}
