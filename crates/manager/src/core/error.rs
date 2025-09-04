use mangadex_api_types_rust::RelationshipType;
use serde::Serialize;
use std::{fmt::Display, num::TryFromIntError, ops::Deref, path::PathBuf, sync::Arc};

use crate::{
    files_dirs::messages::delete::chapter::images::DeleteChapterImagesError,
    history::HistoryBaseError,
};

/// This is the crate Error
#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An std::io::Error captured! \n Details : {0}")]
    Io(#[from] std::io::Error),
    #[error("An Error captured during sending a request \n Details : {0}")]
    ReqwestError(#[from] reqwest::Error),
    #[error("An Error captured from the `mangadex_api` crate \n Details : {0}")]
    MangadexAPIError(#[from] mangadex_api_types_rust::error::Error),
    #[error("An error occured during Joining handles \n Details : {0}")]
    TokioJoinError(#[from] tokio::task::JoinError),
    #[error("An error occured when parsing some string to json \n Details : {0}")]
    SerdeJsonError(#[from] serde_json::Error),
    #[error("An error occured when parsing an uuid \n Details : {0}")]
    UuidError(#[from] uuid::Error),
    #[error("An error occured when parsing bytes to UTF-8 String \n Details : {0}")]
    StringUtf8Error(#[from] std::string::FromUtf8Error),
    #[error("An error occured when parsing bytes to UTF-16 String \n Details : {0}")]
    StringUTF16Error(#[from] std::string::FromUtf16Error),
    #[error("An error occured when parsing something into a String \n Details : {0}")]
    StringParseError(#[from] std::string::ParseError),
    #[error("An error occured when building a mangdex_api request \n Details : {0}")]
    MangadexBuilderError(#[from] mangadex_api_types_rust::error::BuilderError),
    #[error("An Download Tasks limit Exceded {current}/{limit}")]
    DownloadTaskLimitExceded { current: u16, limit: u16 },
    #[error("An error occured when converting into a int")]
    TryIntError(#[from] TryFromIntError),
    #[error("An error occured when sending data between an oneshot channel \n Details: {0}")]
    OneshotRecvError(#[from] tokio::sync::oneshot::error::RecvError),
    #[error("An error occured when acquiring a semaphore \n Details : {0}")]
    AcquireError(#[from] tokio::sync::AcquireError),
    #[error("The file transaction was been roolback due to an error. Details : {0}")]
    RollBacked(String),
    #[error("An RwLock occured \n Details : {0}")]
    RwLockError(#[from] std::sync::PoisonError<String>),
    #[error("We got a {0} mailbox error")]
    MailBox(#[from] actix::MailboxError),
    #[error("the history file for {0} is not found")]
    HistoryFileNotFound(RelationshipType),
    #[error("We got an std thread join error {0}")]
    StdThreadJoin(String),
    #[error("We got an error when manipulation an HistoryEntry: {0}")]
    HistoryBase(#[from] HistoryBaseError),
    #[error("Invalid file entry {0}")]
    InvalidFileName(PathBuf),
    #[error(transparent)]
    WatchRecv(#[from] tokio::sync::watch::error::RecvError),
    #[error("The given task was been cancelled")]
    TaskCanceled,
    #[error("The MangaDexClient is not found")]
    MangaDexClientNotFound,
    #[error("The DirsOption actor is not found")]
    DirsOptionsNotFound,
    #[error("The HistoryService is not found")]
    HistoryServiceNotFound,
    #[error("The initial state can't be sent")]
    NotInitialized,
    #[error(transparent)]
    ApiCore(#[from] api_core::Error),
    #[error(transparent)]
    DeleteChapterImages(#[from] DeleteChapterImagesError),
}

impl Error {
    /// Transform the error into an [`OwnedError`]
    pub fn into_owned(self) -> OwnedError {
        self.into()
    }
    /// Transform the error into an [`ErrorType`]
    pub fn into_type(&self) -> ErrorType {
        self.into()
    }
}

/// This is just [`Error`] wrapped into an [`Arc`],
/// allows you to share the error "safely" between thread
/// since [`Error`] doesn't implement [`Clone`].
///
/// If you don't want the value or want a light-weight alternative to this, use [`ErrorType`] instead
#[derive(Debug, Clone)]
pub struct OwnedError(Arc<Error>);

impl Deref for OwnedError {
    type Target = Arc<Error>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Arc<Error>> for OwnedError {
    fn from(value: Arc<Error>) -> Self {
        Self(value)
    }
}

impl From<Error> for OwnedError {
    fn from(value: Error) -> Self {
        Arc::new(value).into()
    }
}

impl Display for OwnedError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.deref().as_ref().fmt(f)
    }
}

impl std::error::Error for OwnedError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.deref().as_ref().source()
    }
}

/// This is just [`Error`] but without the values.
///
/// You can get it by using [`Into::into`] on [`Error`] or use [`Error::into_type`].
///
/// It can be useful if you just want or share the error without using [`OwnedError`]
#[derive(
    serde::Serialize, Debug, serde::Deserialize, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash,
)]
pub enum ErrorType {
    Io,
    ReqwestError,
    MangadexAPIError,
    TokioJoinError,
    SerdeJsonError,
    UuidError,
    StringUtf8Error,
    StringUTF16Error,
    StringParseError,
    MangadexBuilderError,
    DownloadTaskLimitExceded,
    TryIntError,
    OneshotRecvError,
    AcquireError,
    RollBacked,
    RwLockError,
    MailBox,
    HistoryFileNotFound,
    StdThreadJoin,
    HistoryBase,
    InvalidFileName,
    MissingRelationships,
    DeleteChapterImages,
    WatchRecv,
    TaskCanceled,
    MangaDexClientNotFound,
    HistoryServiceNotFound,
    NotInitialized,
    ApiCore,
    DirsOptionsNotFound,
}

impl From<&Error> for ErrorType {
    fn from(value: &Error) -> Self {
        match value {
            Error::Io(_) => Self::Io,
            Error::ReqwestError(_) => Self::ReqwestError,
            Error::MangadexAPIError(_) => Self::MangadexAPIError,
            Error::TokioJoinError(_) => Self::TokioJoinError,
            Error::SerdeJsonError(_) => Self::SerdeJsonError,
            Error::UuidError(_) => Self::UuidError,
            Error::StringUtf8Error(_) => Self::StringUtf8Error,
            Error::StringUTF16Error(_) => Self::StringUTF16Error,
            Error::StringParseError(_) => Self::StringParseError,
            Error::MangadexBuilderError(_) => Self::MangadexBuilderError,
            Error::DownloadTaskLimitExceded { .. } => Self::DownloadTaskLimitExceded,
            Error::TryIntError(_) => Self::TryIntError,
            Error::OneshotRecvError(_) => Self::OneshotRecvError,
            Error::AcquireError(_) => Self::AcquireError,
            Error::RollBacked(_) => Self::RollBacked,
            Error::RwLockError(_) => Self::RwLockError,
            Error::MailBox(_) => Self::MailBox,
            Error::HistoryFileNotFound(_) => Self::HistoryFileNotFound,
            Error::StdThreadJoin(_) => Self::StdThreadJoin,
            Error::HistoryBase(_) => Self::HistoryBase,
            Error::InvalidFileName(_) => Self::InvalidFileName,
            Error::WatchRecv(_) => Self::WatchRecv,
            Error::TaskCanceled => Self::TaskCanceled,
            Error::MangaDexClientNotFound => Self::MangaDexClientNotFound,
            Error::DirsOptionsNotFound => Self::DirsOptionsNotFound,
            Error::HistoryServiceNotFound => Self::HistoryServiceNotFound,
            Error::NotInitialized => Self::NotInitialized,
            Error::ApiCore(_) => Self::ApiCore,
            Error::DeleteChapterImages(_) => Self::DeleteChapterImages,
        }
    }
}

impl From<Error> for ErrorType {
    fn from(value: Error) -> Self {
        (&value).into()
    }
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
