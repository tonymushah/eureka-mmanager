pub mod chapter;
pub mod cover;
pub mod filter;
pub mod manga;
pub mod random;
pub mod results;
pub mod sort;

pub use filter::{IntoFiltered, IntoParamedFilteredStream};
pub use random::{AsyncRand, Rand};
pub use results::{AsyncPaginate, Paginate};
pub use sort::{AsyncIntoSorted, IntoSorted};

use crate::ManagerCoreResult;

pub trait Pull<T, I> {
    fn pull(&self, id: I) -> ManagerCoreResult<T>;
}

pub trait PartialRelated<T> {
    fn prt_rlted(&self, data: &T) -> Option<bool>;
}
pub trait Related<T> {
    fn rlted(&self, data: &T) -> bool;
}

impl<T, S> PartialRelated<T> for S
where
    S: Related<T>,
{
    fn prt_rlted(&self, data: &T) -> Option<bool> {
        Some(self.rlted(data))
    }
}

pub trait AsyncRelated<T> {
    type Error;
    fn is_async_related(
        &self,
        data: &T,
    ) -> impl std::future::Future<Output = Result<bool, <Self as AsyncRelated<T>>::Error>> + Send;
}

/// Guess if the input value is valid to the host struct
///
/// Can be used to filter stream, or iterators
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
