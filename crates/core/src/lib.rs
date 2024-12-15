//! # eureka_mmanager-core
//!
//! ## This library is still in developpement and not yet documented. Use it at your risk

#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod data_pulls;
pub mod data_push;
pub mod error;
pub mod file_dirs;

pub(crate) type ManagerCoreResult<T, E = error::Error> = Result<T, E>;

pub use error::Error;
pub use file_dirs::DirsOptions;
