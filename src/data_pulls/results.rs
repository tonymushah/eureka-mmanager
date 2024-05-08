use std::num::TryFromIntError;

use mangadex_api_schema_rust::v5::Results;
use mangadex_api_types_rust::{ResponseType, ResultType};
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection<T> {
    pub data: Vec<T>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
}

impl<T> Collection<T> {
    pub fn into_results(self) -> Result<Results<T>, TryFromIntError> {
        self.try_into()
    }
}

impl<T> TryFrom<Collection<T>> for Results<T> {
    type Error = TryFromIntError;
    fn try_from(value: Collection<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            result: ResultType::Ok,
            response: ResponseType::Collection,
            data: value.data,
            limit: value.limit.try_into()?,
            offset: value.offset.try_into()?,
            total: value.total.try_into()?,
        })
    }
}

pub trait AsyncPaginate<T> {
    fn paginate(
        self,
        offset: usize,
        limit: usize,
    ) -> impl std::future::Future<Output = Collection<T>> + Send;
}

pub trait Paginate<T> {
    fn paginate(self, offset: usize, limit: usize) -> Collection<T>;
}

impl<T> Paginate<T> for Vec<T> {
    fn paginate(self, offset: usize, limit: usize) -> Collection<T> {
        let total = self.len();
        let data = self
            .into_iter()
            .skip(offset)
            .take(limit)
            .collect::<Vec<_>>();
        Collection {
            total,
            data,
            offset,
            limit,
        }
    }
}

impl<S, T> AsyncPaginate<T> for S
where
    S: Stream<Item = T> + Send,
    T: Send,
{
    async fn paginate(self, offset: usize, limit: usize) -> Collection<T> {
        let stream = Box::pin(self);
        let (_, pre_total) = stream.size_hint();
        if let Some(total) = pre_total {
            let data = stream.skip(offset).take(limit).collect::<Vec<_>>().await;
            Collection {
                data,
                offset,
                limit,
                total,
            }
        } else {
            stream.collect::<Vec<_>>().await.paginate(offset, limit)
        }
    }
}
