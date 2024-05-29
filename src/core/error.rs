use mangadex_api_types_rust::RelationshipType;
use serde::Serialize;
use std::{fmt::Display, num::TryFromIntError, ops::Deref, path::PathBuf, sync::Arc};

use crate::{
    files_dirs::messages::delete::chapter::images::DeleteChapterImagesError,
    history::HistoryBaseError,
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
pub struct WhenError {
    type_: ErrorType,
    message: String,
    result: String,
}

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
    #[error("The {0} doesn't exist")]
    DirsOptionsVerification(#[from] DirsOptionsVerificationError),
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
    #[error("Error when deserializing a .cbor file {0}")]
    CiboriumDeIo(#[from] ciborium::de::Error<std::io::Error>),
    #[error("Error when serializing a .cbor file {0}")]
    CiboriumSerIo(#[from] ciborium::ser::Error<std::io::Error>),
    #[error("Regex error {0}")]
    Regex(#[from] regex::Error),
    #[error("Missing Relationship {0:#?}")]
    MissingRelationships(Vec<RelationshipType>),
    #[error(transparent)]
    DeleteChapterImages(#[from] DeleteChapterImagesError),
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
}

impl Error {
    pub fn into_owned(self) -> OwnedError {
        self.into()
    }
}

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

#[derive(Debug, thiserror::Error)]
pub enum DirsOptionsVerificationError {
    #[error("The data dir doesn:t exist")]
    DataRoot,
    #[error("The history dir doesn:t exist")]
    History,
    #[error("The chapters dir doesn:t exist")]
    Chapters,
    #[error("The covers dir doesn:t exist")]
    Covers,
    #[error("The covers images dir doesn:t exist")]
    CoverImages,
    #[error("The mangas dir doesn't exist")]
    Mangas,
}

#[derive(serde::Serialize, Debug, serde::Deserialize)]
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
    Other,
    MangadexBuilderError,
    DownloadTaskLimitExceded,
    TryIntError,
    OneshotRecvError,
    AcquireError,
    RollBacked,
    RwLockError,
}

impl Serialize for Error {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.to_string().as_str())
    }
}
