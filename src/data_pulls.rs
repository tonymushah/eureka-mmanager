pub mod manga;

pub trait Related<T> {
    fn is_related(&self, data: &T) -> bool;
}

pub trait ParatialRelated<T> {
    fn partial_related(&self, data: &T) -> Option<bool>;
}

#[async_trait::async_trait]
pub trait AsyncRelated<T> {
    type Error;
    async fn is_async_related(&self, data: &T) -> Result<bool, <Self as AsyncRelated<T>>::Error>;
}

pub trait Validate<T> {
    fn is_valid(&self, input: &T) -> bool;
}
