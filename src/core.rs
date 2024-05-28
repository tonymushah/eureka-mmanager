mod error;

pub use error::{DirsOptionsVerificationError, Error, ErrorType, OwnedError};

pub type ManagerCoreResult<T> = Result<T, Error>;
