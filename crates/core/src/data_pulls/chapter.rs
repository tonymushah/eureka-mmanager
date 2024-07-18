pub mod filter;
pub mod ids;
pub mod images;
pub mod list;

use std::{cmp::Ordering, fs::File, io::BufReader};

pub use filter::ChapterListDataPullFilterParams;

use ids::ChapterIdsListDataPull;
use list::ChapterListDataPull;
use mangadex_api_schema_rust::v5::{ChapterData, ChapterObject};
use mangadex_api_types_rust::{ChapterSortOrder, OrderDirection};
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::{DirsOptions, ManagerCoreResult};

use super::{sort::IntoSorted, AsyncIntoSorted, IntoFiltered, IntoParamedFilteredStream, Pull};

impl<S> AsyncIntoSorted<ChapterSortOrder> for S
where
    S: Stream<Item = ChapterObject> + Send,
{
    type Item = ChapterObject;
    async fn to_sorted(
        self,
        params: ChapterSortOrder,
    ) -> Vec<<Self as AsyncIntoSorted<ChapterSortOrder>>::Item> {
        let stream = Box::pin(self);
        stream
            .collect::<Vec<ChapterObject>>()
            .await
            .to_sorted(params)
    }
}

impl IntoSorted<ChapterSortOrder> for Vec<ChapterObject> {
    type Item = ChapterObject;
    fn to_sorted(
        mut self,
        params: ChapterSortOrder,
    ) -> Vec<<Self as IntoSorted<ChapterSortOrder>>::Item> {
        match params {
            ChapterSortOrder::CreatedAt(o) => match o {
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
            ChapterSortOrder::Chapter(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a
                            .attributes
                            .chapter
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        let b = b
                            .attributes
                            .chapter
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a
                            .attributes
                            .chapter
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        let b = b
                            .attributes
                            .chapter
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        b.partial_cmp(&a).unwrap_or(Ordering::Equal)
                    });
                }
            },
            ChapterSortOrder::PublishAt(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.publish_at.as_ref().map(|d| d.as_ref());
                        let b = b.attributes.publish_at.as_ref().map(|d| d.as_ref());
                        a.cmp(&b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.publish_at.as_ref().map(|d| d.as_ref());
                        let b = b.attributes.publish_at.as_ref().map(|d| d.as_ref());
                        b.cmp(&a)
                    });
                }
            },
            ChapterSortOrder::ReadableAt(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.readable_at.as_ref().map(|d| d.as_ref());
                        let b = b.attributes.readable_at.as_ref().map(|d| d.as_ref());
                        a.cmp(&b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.readable_at.as_ref().map(|d| d.as_ref());
                        let b = b.attributes.readable_at.as_ref().map(|d| d.as_ref());
                        b.cmp(&a)
                    });
                }
            },
            ChapterSortOrder::UpdatedAt(o) => match o {
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
            ChapterSortOrder::Volume(o) => match o {
                OrderDirection::Ascending => {
                    self.sort_by(|a, b| {
                        let a = a
                            .attributes
                            .volume
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        let b = b
                            .attributes
                            .volume
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        a.partial_cmp(&b).unwrap_or(Ordering::Equal)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a
                            .attributes
                            .volume
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        let b = b
                            .attributes
                            .volume
                            .as_ref()
                            .and_then(|c| -> Option<f32> { c.parse::<f32>().ok() });
                        b.partial_cmp(&a).unwrap_or(Ordering::Equal)
                    });
                }
            },
            _ => {}
        };
        self
    }
}

impl<S> IntoParamedFilteredStream<ChapterListDataPullFilterParams> for S where
    S: Stream<Item = ChapterObject>
{
}

impl<I> IntoFiltered<ChapterListDataPullFilterParams> for I where I: Iterator<Item = ChapterObject> {}

impl Pull<ChapterObject, Uuid> for DirsOptions {
    type Error = crate::Error;
    fn pull(&self, id: Uuid) -> crate::ManagerCoreResult<ChapterObject> {
        let manga_id_path = self.chapters_add(format!("{}", id)).join("data.json");
        let file = BufReader::new(File::open(manga_id_path)?);
        let manga: ChapterData = serde_json::from_reader(file)?;
        Ok(manga.data)
    }
}

impl DirsOptions {
    pub fn pull_all_chapter(&self) -> ManagerCoreResult<ChapterListDataPull> {
        ChapterListDataPull::new(self.chapters_add(""))
    }
    pub fn pull_chapter_ids(&self, ids: Vec<Uuid>) -> ChapterIdsListDataPull {
        ChapterIdsListDataPull::new(self.chapters.clone(), ids)
    }
}
