pub mod filter;
pub mod ids;
pub mod list;

use std::cmp::Ordering;

use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::{CoverSortOrder, OrderDirection};
use tokio_stream::{Stream, StreamExt};

use super::{sort::IntoSorted, AsyncIntoSorted, IntoParamedFilteredStream};
use filter::CoverListDataPullFilterParams;

impl<S> AsyncIntoSorted<CoverSortOrder> for S
where
    S: Stream<Item = CoverObject> + Send,
{
    type Item = CoverObject;
    async fn to_sorted(
        self,
        params: CoverSortOrder,
    ) -> Vec<<Self as AsyncIntoSorted<CoverSortOrder>>::Item> {
        let stream = Box::pin(self);
        stream.collect::<Vec<CoverObject>>().await.to_sorted(params)
    }
}

impl IntoSorted<CoverSortOrder> for Vec<CoverObject> {
    type Item = CoverObject;
    fn to_sorted(
        mut self,
        params: CoverSortOrder,
    ) -> Vec<<Self as IntoSorted<CoverSortOrder>>::Item> {
        match params {
            CoverSortOrder::CreatedAt(o) => match o {
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
            CoverSortOrder::UpdatedAt(o) => match o {
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
            CoverSortOrder::Volume(o) => match o {
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
        }
        self
    }
}

impl<S> IntoParamedFilteredStream<CoverListDataPullFilterParams> for S where
    S: Stream<Item = CoverObject>
{
}