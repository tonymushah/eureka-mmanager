mod error;

pub use error::Error;

pub type ManagerCoreResult<T> = Result<T, Error>; 