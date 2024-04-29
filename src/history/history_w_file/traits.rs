use crate::history::{Insert, Remove};

pub trait Commitable {
    type Output;
    fn commit(&mut self) -> Self::Output;
}

pub trait RollBackable {
    type Output;
    fn rollback(&mut self) -> Self::Output;
}

pub trait AutoCommitRollbackInsert<T>: Commitable + RollBackable + Insert<T> {
    type Output;
    fn insert(&mut self, input: T) -> <Self as AutoCommitRollbackInsert<T>>::Output;
}

pub trait AutoCommitRollbackRemove<T>: Commitable + RollBackable + Remove<T> {
    type Output;
    fn remove(&mut self, input: T) -> <Self as AutoCommitRollbackRemove<T>>::Output;
}

#[async_trait::async_trait]
pub trait AsyncCommitable {
    type Output;
    async fn commit(&mut self) -> Self::Output;
}

#[async_trait::async_trait]
pub trait AsyncRollBackable {
    type Output;
    async fn rollback(&mut self) -> Self::Output;
}

#[async_trait::async_trait]
pub trait AsyncAutoCommitRollbackInsert<'a, T> {
    type Output;
    async fn insert(&'a mut self, input: T) -> <Self as AsyncAutoCommitRollbackInsert<T>>::Output;
}

#[async_trait::async_trait]
pub trait AsyncAutoCommitRollbackRemove<'a, T> {
    type Output;
    async fn remove(&'a mut self, input: T) -> <Self as AsyncAutoCommitRollbackRemove<T>>::Output;
}

#[async_trait::async_trait]
pub trait AsyncCommitableWInput<'a, T> {
    type Output;
    async fn commit(&'a mut self, input: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait AsyncRollBackableWInput<'a, T> {
    type Output;
    async fn rollback(&'a mut self, input: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait NoLFAsyncAutoCommitRollbackInsert<T> {
    type Output;
    async fn insert(&mut self, input: T) -> Self::Output;
}

#[async_trait::async_trait]
pub trait NoLFAsyncAutoCommitRollbackRemove<T> {
    type Output;
    async fn remove(&mut self, input: T) -> Self::Output;
}
