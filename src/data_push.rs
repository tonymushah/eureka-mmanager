pub mod chapter;

use crate::ManagerCoreResult;

pub trait Push<T> {
    fn push(&mut self, data: T) -> ManagerCoreResult<()>;
}
