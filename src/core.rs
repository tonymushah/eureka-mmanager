mod error;

pub use error::{Error, ErrorType};

pub type ManagerCoreResult<T> = Result<T, Error>;
