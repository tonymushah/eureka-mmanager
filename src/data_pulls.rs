pub mod manga;
pub mod sort;

pub use sort::{AsyncIntoSorted, IntoSorted};

pub trait Related<T> {
    fn is_related(&self, data: &T) -> bool;
}

pub trait ParatialRelated<T> {
    fn partial_related(&self, data: &T) -> Option<bool>;
}

pub trait AsyncRelated<T> {
    type Error;
    fn is_async_related(
        &self,
        data: &T,
    ) -> impl std::future::Future<Output = Result<bool, <Self as AsyncRelated<T>>::Error>> + Send;
}

pub trait Validate<T> {
    fn is_valid(&self, input: &T) -> bool;
}
