pub mod chapter;
pub mod cover;
pub mod manga;

use crate::ManagerCoreResult;

pub trait Push<T> {
    fn push(&mut self, data: T) -> ManagerCoreResult<()>;
    fn verify_and_push(&mut self, data: T) -> ManagerCoreResult<()> {
        self.push(data)
    }
}
