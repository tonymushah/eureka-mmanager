pub mod chapter;
pub mod cover;
pub mod filter;
pub mod manga;
pub mod random;
pub mod results;
pub mod sort;

use std::ops::Deref;

pub use filter::{IntoFiltered, IntoParamedFilteredStream};
pub use random::{AsyncRand, Rand};
pub use results::{AsyncPaginate, Paginate};
pub use sort::{AsyncIntoSorted, IntoSorted};

use crate::{DirsOptions, ManagerCoreResult};

#[derive(Debug)]
pub struct DataPull<'a>(&'a DirsOptions);

impl<'a> Deref for DataPull<'a> {
    type Target = DirsOptions;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl DirsOptions {
    pub fn data_pull(&self) -> DataPull<'_> {
        DataPull(self)
    }
}

pub trait Pull<T, I> {
    fn pull(&self, id: I) -> ManagerCoreResult<T>;
}

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

#[macro_export]
macro_rules! option_bool_match {
    ($t:expr) => {
        match $t {
            Some(o) => o,
            None => return Some(false),
        }
    };
}
