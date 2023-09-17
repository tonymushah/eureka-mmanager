mod entry;
pub use entry::HistoryEntry;

mod history;
pub use history::History;

pub mod history_w_file;
pub use history_w_file::HistoryWFile;

pub trait Insert<T> {
    type Output;
    fn insert(&mut self, input : T) -> Self::Output;
}

pub trait Remove<T> {
    type Output;
    fn remove(&mut self, input : T) -> Self::Output;
}

pub trait IsIn<T>{
    type Output;
    fn is_in(&self, to_use : T) -> Self::Output;
}