pub mod error;
pub mod file_dirs;

pub(crate) type ManagerCoreResult<T, E = error::Error> = Result<T, E>;
