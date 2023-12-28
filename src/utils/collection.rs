use std::num::TryFromIntError;

use mangadex_api_schema_rust::v5::Results;
use serde::{Deserialize, Serialize};
use tokio_stream::{Stream, StreamExt};

use crate::core::ManagerCoreResult;

#[derive(Clone, Serialize)]
pub struct Collection<T>
where
    T: Serialize,
    T: Clone,
{
    data: Vec<T>,
    limit: usize,
    offset: usize,
    total: usize,
}

impl<T> Collection<T>
where
    T: Serialize,
    T: Clone,
{
    pub fn new(
        to_use: &mut Vec<T>,
        limit: usize,
        offset: usize,
    ) -> ManagerCoreResult<Collection<T>> {
        if offset > to_use.len() {
            ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "the offset is greater than the vector length",
            )))
        } else {
            let (_, right) = to_use.split_at(offset);
            let data;
            if right.len() <= limit {
                data = right.to_vec();
                Ok(Collection {
                    data,
                    limit,
                    offset,
                    total: to_use.len(),
                })
            } else {
                let (left1, _) = right.split_at(limit);
                data = left1.to_vec();
                Ok(Collection {
                    data,
                    limit,
                    offset,
                    total: to_use.len(),
                })
            }
        }
    }
    pub fn get_data(&self) -> &Vec<T> {
        &self.data
    }
    pub fn get_total(&self) -> usize {
        self.total
    }
    pub fn get_offset(&self) -> usize {
        self.offset
    }
    pub fn get_limit(&self) -> usize {
        self.limit
    }
    pub fn convert_to<S, F>(&self, f: F) -> ManagerCoreResult<Collection<S>>
    where
        F: Fn(T) -> S,
        S: Clone,
        S: serde::Serialize,
    {
        ManagerCoreResult::Ok(Collection {
            data: self.data.iter().cloned().map(f).collect(),
            offset: self.offset,
            limit: self.limit,
            total: self.total,
        })
    }
    pub async fn from_async_stream<S>(
        stream: S,
        limit: usize,
        offset: usize,
    ) -> ManagerCoreResult<Collection<T>>
    where
        S: Stream<Item = T>,
    {
        let stream = stream;
        tokio::pin!(stream);
        let mut to_use: Vec<T> = stream.collect().await;
        Self::new(&mut to_use, limit, offset)
    }
}

impl<'de, T> TryFrom<Collection<T>> for Results<T>
where
    T: Serialize + Deserialize<'de> + Clone,
{
    type Error = TryFromIntError;
    fn try_from(value: Collection<T>) -> Result<Self, Self::Error> {
        Ok(Self {
            result: mangadex_api_types_rust::ResultType::Ok,
            response: mangadex_api_types_rust::ResponseType::Collection,
            data: value.data,
            limit: value.limit.try_into()?,
            offset: value.offset.try_into()?,
            total: value.total.try_into()?,
        })
    }
}
