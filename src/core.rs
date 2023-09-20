mod error;

pub use error::{Error, ErrorType, WhenError};

pub type ManagerCoreResult<T> = Result<T, Error>;
