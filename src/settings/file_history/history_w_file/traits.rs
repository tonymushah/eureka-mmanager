use crate::settings::file_history::{Insert, Remove};

pub trait Commitable {
    type Output;
    fn commit(&mut self) -> Self::Output;
}

pub trait RollBackable {
    type Output;
    fn rollback(&mut self) -> Self::Output;
}

pub trait AutoCommitRollbackInsert<T> : Commitable + RollBackable + Insert<T> {
    type Output;
    fn insert(&mut self, input : T) -> <Self as crate::settings::file_history::history_w_file::traits::AutoCommitRollbackInsert<T>>::Output;
}

pub trait AutoCommitRollbackRemove<T> : Commitable + RollBackable + Remove<T>  {
    type Output;
    fn remove(&mut self, input : T) -> <Self as crate::settings::file_history::history_w_file::traits::AutoCommitRollbackRemove<T>>::Output;
}