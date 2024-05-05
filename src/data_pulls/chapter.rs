pub mod ids;
pub mod list;

use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::{ChapterSortOrder, OrderDirection};
use tokio_stream::{Stream, StreamExt};

use super::{sort::IntoSorted, AsyncIntoSorted};

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
                        let a = a.attributes.chapter.as_ref();
                        let b = b.attributes.chapter.as_ref();
                        a.cmp(&b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.chapter.as_ref();
                        let b = b.attributes.chapter.as_ref();
                        b.cmp(&a)
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
                        let a = a.attributes.volume.as_ref();
                        let b = b.attributes.volume.as_ref();
                        a.cmp(&b)
                    });
                }
                OrderDirection::Descending => {
                    self.sort_by(|a, b| {
                        let a = a.attributes.volume.as_ref();
                        let b = b.attributes.volume.as_ref();
                        b.cmp(&a)
                    });
                }
            },
            _ => {}
        };
        self
    }
}
