use tokio_stream::{Stream, StreamExt};

use crate::core::ManagerCoreResult;

#[derive(Clone, serde::Serialize)]
pub struct Collection<T>
where
    T: serde::Serialize,
    T: Clone,
{
    data: Vec<T>,
    limit: usize,
    offset: usize,
    total: usize,
}

impl<T> Collection<T>
where
    T: serde::Serialize,
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
    pub fn get_data(self) -> Vec<T> {
        self.data
    }
    pub fn get_total(self) -> usize {
        self.total
    }
    pub fn get_offset(self) -> usize {
        self.offset
    }
    pub fn get_limit(self) -> usize {
        self.limit
    }
    pub fn convert_to<S, F>(&mut self, f: F) -> ManagerCoreResult<Collection<S>>
    where
        F: Fn(T) -> S,
        S: Clone,
        S: serde::Serialize,
    {
        let mut new_data: Vec<S> = Vec::new();
        let old_data = self.data.clone();
        for data in old_data {
            new_data.push(f(data));
        }
        ManagerCoreResult::Ok(Collection {
            data: new_data,
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
