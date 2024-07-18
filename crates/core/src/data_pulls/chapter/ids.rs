use std::{fs::File, io::BufReader, path::PathBuf, vec::IntoIter};

use mangadex_api_schema_rust::v5::{ChapterData, ChapterObject};
#[cfg(feature = "stream")]
use std::task::Poll;
#[cfg(feature = "stream")]
use tokio_stream::Stream;
use uuid::Uuid;

use crate::ManagerCoreResult;

#[derive(Debug)]
#[cfg_attr(feature = "actix", actix::MessageResponse)]
pub struct ChapterIdsListDataPull {
    chapter_path: PathBuf,
    iter: IntoIter<Uuid>,
}

impl ChapterIdsListDataPull {
    pub(crate) fn new(chapter_path: PathBuf, ids: Vec<Uuid>) -> Self {
        Self {
            chapter_path,
            iter: ids.into_iter(),
        }
    }
    // TODO add cbor support
    fn id_to_chapter(&self, entry: Uuid) -> ManagerCoreResult<ChapterObject> {
        let entry = self.chapter_path.join(format!("{entry}"));
        if !entry.exists() || !entry.is_dir() || !entry.join("data.json").exists() {
            return Err(crate::Error::InvalidFileName(entry));
        }
        let file = BufReader::new(File::open(entry.join("data.json"))?);
        let o: ChapterData = serde_json::from_reader(file)?;
        Ok(o.data)
    }
}

impl Iterator for ChapterIdsListDataPull {
    type Item = ManagerCoreResult<ChapterObject>;
    fn next(&mut self) -> Option<Self::Item> {
        let next = self.iter.next()?;
        Some(self.id_to_chapter(next))
    }
}

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl Stream for ChapterIdsListDataPull {
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
