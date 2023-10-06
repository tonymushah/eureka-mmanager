mod entry;
pub use entry::HistoryEntry;

mod history;
pub use history::History;

pub mod history_w_file;
pub use history_w_file::HistoryWFile;

pub trait Insert<T> {
    type Output;
    fn insert(&mut self, input: T) -> Self::Output;
}

pub trait Remove<T> {
    type Output;
    fn remove(&mut self, input: T) -> Self::Output;
}

pub trait IsIn<T> {
    type Output;
    fn is_in(&self, to_use: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait AsyncInsert<'a, T> {
    type Output;
    async fn insert(&'a mut self, input: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait AsyncRemove<'a, T> {
    type Output;
    async fn remove(&'a mut self, input: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait AsyncIsIn<'a, T> {
    type Output;
    async fn is_in(&'a self, to_use: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait NoLFAsyncIsIn<T> {
    type Output;
    async fn is_in(&self, to_use: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait NoLFAsyncInsert<T> {
    type Output;
    async fn insert(&mut self, input: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait NoLFAsyncRemove<T> {
    type Output;
    async fn remove(&mut self, input: T) -> Self::Output;
}
