pub use crate::r#core::ManagerCoreResult;

pub mod data_pulls;
pub mod extractors;
pub mod files_dirs;
pub mod history;

mod r#core;

pub use crate::r#core::{DirsOptionsVerificationError, Error, ErrorType};

pub use files_dirs::{
    messages::{
        join_chapters::JoinChaptersMessage, join_covers::JoinCoversMessage,
        join_covers_images::JoinCoversImagesMessage, join_data::JoinDataMessage,
        join_history::JoinHistoryMessage, modify_chapters_path::ModifyChaptersDirMessage,
        modify_covers_path::ModifyCoversDirMessage, modify_data_path::ModifyDataDirMessage,
    },
    DirsOptions,
};
