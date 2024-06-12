pub use crate::r#core::ManagerCoreResult;

pub mod data_pulls;
pub mod data_push;
pub mod download;
pub mod extractors;
pub mod files_dirs;
pub mod history;

type MailBoxResult<T, E = actix::MailboxError> = Result<T, E>;

mod r#core;

pub use crate::r#core::{DirsOptionsVerificationError, Error, ErrorType, OwnedError};

pub use files_dirs::{
    messages::{
        join::{
            JoinChaptersMessage, JoinCoversImagesMessage, JoinCoversMessage, JoinDataMessage,
            JoinHistoryMessage,
        },
        modify::{ModifyChaptersDirMessage, ModifyCoversDirMessage, ModifyDataDirMessage},
    },
    DirsOptions,
};
