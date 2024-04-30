pub use crate::r#core::ManagerCoreResult;

pub mod files_dirs;
pub mod history;

mod r#core;

pub use crate::r#core::{DirsOptionsVerificationError, Error, ErrorType};

pub mod r#static;
