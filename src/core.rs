mod error;

pub use error::{DirsOptionsVerificationError, Error, ErrorType};

pub type ManagerCoreResult<T> = Result<T, Error>;
