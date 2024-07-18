use std::num::TryFromIntError;

use mangadex_api_schema_rust::v5::Results;
use mangadex_api_types_rust::{ResponseType, ResultType};
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt};

/// The result of [`Paginate::paginate`] or [`AsyncPaginate::paginate`].
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Collection<T> {
    pub data: Vec<T>,
    pub offset: usize,
    pub limit: usize,
    pub total: usize,
}

impl<T> Collection<T> {
    /// Transform this into an [`Results`].
    ///
    /// __Why it returns a [`Result<Results<T>, TryFromIntError>`]?__
    ///
    /// Since [`Results::offset`], [`Results::total`], [`Results::limit`], [`Results::total`] is not [`usize`], we need to transform them into an [`u32`].
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

/// Paginate allows you to get portion of the underlying stream.
///
/// This is heavily inspired by the [SQL offset limit system](https://www.postgresql.org/docs/current/queries-limit.html).
///
/// If you want an synchronous version, use [`Paginate`] instead.
///
/// Note: If the underlying [`Stream::size_hint`] doesn't give the total, it will collect the stream into a [`Vec`] and call [`Paginate::paginate`] after.
pub trait AsyncPaginate<T> {
    fn paginate(
        self,
        offset: usize,
        limit: usize,
    ) -> impl std::future::Future<Output = Collection<T>> + Send;
}

/// Paginate allows you to get portion of the underlying [`Vec`] (unfortunalty).
///
/// This is heavily inspired by the [SQL offset limit system](https://www.postgresql.org/docs/current/queries-limit.html).
///
/// If you want an asynchronous version, use [`Paginate`] instead.
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
