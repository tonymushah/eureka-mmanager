mod error;

pub use error::{Error, ErrorType, OwnedError};

pub type ManagerCoreResult<T> = Result<T, Error>;
