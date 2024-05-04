use crate::history::{Insert, Remove};

pub trait Commitable {
    type Output;
    fn commit(&self) -> Self::Output;
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

pub trait AsyncCommitable {
    type Output;
    fn commit(&self) -> impl std::future::Future<Output = Self::Output> + Send;
}

pub trait AsyncRollBackable {
    type Output;
    fn rollback(&mut self) -> impl std::future::Future<Output = Self::Output> + Send;
}

pub trait AsyncAutoCommitRollbackInsert<'a, T> {
    type Output;
    fn insert(
        &'a mut self,
        input: T,
    ) -> impl std::future::Future<Output = <Self as AsyncAutoCommitRollbackInsert<T>>::Output> + Send;
}

pub trait AsyncAutoCommitRollbackRemove<'a, T> {
    type Output;
    fn remove(
        &'a mut self,
        input: T,
    ) -> impl std::future::Future<Output = <Self as AsyncAutoCommitRollbackRemove<T>>::Output> + Send;
}
