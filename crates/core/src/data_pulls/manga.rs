pub mod aggregate;
pub mod filter;
pub mod ids;
pub mod list;

use std::{fs::File, io::BufReader};

pub use filter::MangaListDataPullFilterParams;
pub use ids::MangaIdsListDataPull;
pub use list::MangaListDataPull;
use mangadex_api_schema_rust::v5::{MangaData, MangaObject};
use mangadex_api_types_rust::{MangaSortOrder, OrderDirection};
#[cfg(feature = "stream")]
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

use super::{sort::IntoSorted, IntoFiltered, Pull};
#[cfg(feature = "stream")]
use super::{AsyncIntoSorted, IntoParamedFilteredStream};

#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S> AsyncIntoSorted<MangaSortOrder> for S
where
    S: Stream<Item = MangaObject> + Send,
{
    type Item = MangaObject;
    async fn to_sorted(
        self,
        params: MangaSortOrder,
    ) -> Vec<<Self as AsyncIntoSorted<MangaSortOrder>>::Item> {
        let stream = Box::pin(self);
        stream.collect::<Vec<MangaObject>>().await.to_sorted(params)
    }
}

impl IntoSorted<MangaSortOrder> for Vec<MangaObject> {
    type Item = MangaObject;
    fn to_sorted(
        mut self,
        params: MangaSortOrder,
    ) -> Vec<<Self as IntoSorted<MangaSortOrder>>::Item> {
        match params {
            MangaSortOrder::CreatedAt(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.created_at.as_ref();
                        let b = b.attributes.created_at.as_ref();
                        a.cmp(b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.created_at.as_ref();
                        let b = b.attributes.created_at.as_ref();
                        b.cmp(a)
                    });
                }
            },
            MangaSortOrder::FollowedCount(_) => {}
            MangaSortOrder::LatestUploadedChapter(_) => {}
            MangaSortOrder::Relevance(_) => {}
            MangaSortOrder::Title(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.title.values().collect::<Vec<_>>();
                        let b = b.attributes.title.values().collect::<Vec<_>>();
                        a.cmp(&b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.title.values().collect::<Vec<_>>();
                        let b = b.attributes.title.values().collect::<Vec<_>>();
                        b.cmp(&a)
                    });
                }
            },
            MangaSortOrder::UpdatedAt(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.updated_at.as_ref().map(|d| d.as_ref());
                        let b = b.attributes.updated_at.as_ref().map(|d| d.as_ref());
                        a.cmp(&b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.updated_at.as_ref().map(|d| d.as_ref());
                        let b = b.attributes.updated_at.as_ref().map(|d| d.as_ref());
                        b.cmp(&a)
                    });
                }
            },
            MangaSortOrder::Year(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.year.as_ref();
                        let b = b.attributes.year.as_ref();
                        a.cmp(&b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.year.as_ref();
                        let b = b.attributes.year.as_ref();
                        b.cmp(&a)
                    });
                }
            },
            _ => {}
        };
        self
    }
}

#[cfg(feature = "stream")]
impl<S> IntoParamedFilteredStream<MangaListDataPullFilterParams> for S where
    S: Stream<Item = MangaObject>
{
}

impl<I> IntoFiltered<MangaListDataPullFilterParams> for I where I: Iterator<Item = MangaObject> {}

impl Pull<MangaObject, Uuid> for DirsOptions {
    type Error = crate::Error;
    // TODO add cbor support
    fn pull(&self, id: Uuid) -> crate::ManagerCoreResult<MangaObject> {
        let manga_id_path = self.mangas_add(format!("{}.json", id));
        let file = BufReader::new(File::open(manga_id_path)?);
        let manga: MangaData = serde_json::from_reader(file)?;
        Ok(manga.data)
    }
}

impl DirsOptions {
    pub fn pull_all_mangas(&self) -> ManagerCoreResult<MangaListDataPull> {
        MangaListDataPull::new(self.mangas.clone())
    }
    pub fn pull_mangas_ids(&self, ids: Vec<Uuid>) -> MangaIdsListDataPull {
        MangaIdsListDataPull::new(self.covers.clone(), ids)
    }
}
