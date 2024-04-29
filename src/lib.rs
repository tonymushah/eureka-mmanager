pub use crate::r#core::ManagerCoreResult;

pub mod settings;

mod r#core;

pub use crate::r#core::{DirsOptionsVerificationError, Error, ErrorType};

pub mod r#static;
