mod entry;
pub use entry::HistoryEntry;

mod base;
pub use base::error::HistoryBaseError;
pub use base::HistoryBase;

pub mod history_w_file;
pub use history_w_file::HistoryWFile;

pub mod service;

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

pub trait IsInMut<'a, T> {
    type Output;
    fn is_in(&'a mut self, to_use: T) -> Self::Output;
}

pub trait AsyncInsert<'a, T> {
    type Output;
    fn insert(&'a mut self, input: T) -> impl std::future::Future<Output = Self::Output> + Send;
}

pub trait AsyncRemove<'a, T> {
    type Output;
    fn remove(&'a mut self, input: T) -> impl std::future::Future<Output = Self::Output> + Send;
}

pub trait AsyncIsIn<'a, T> {
    type Output;
    fn is_in(&'a self, to_use: T) -> impl std::future::Future<Output = Self::Output> + Send;
}

pub trait NoLFAsyncIsIn<T> {
    type Output;
    fn is_in(&self, to_use: T) -> impl std::future::Future<Output = Self::Output> + Send;
}

pub trait NoLFAsyncInsert<T> {
    type Output;
    fn insert(&mut self, input: T) -> impl std::future::Future<Output = Self::Output> + Send;
}

pub trait NoLFAsyncRemove<T> {
    type Output;
    fn remove(&mut self, input: T) -> impl std::future::Future<Output = Self::Output> + Send;
}
