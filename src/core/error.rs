use std::num::TryFromIntError;
#[cfg(feature = "actix_web")]
use actix_web::ResponseError;
use serde::Serialize;

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
    #[error(transparent)]
    Other(#[from] anyhow::Error),
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

#[cfg(feature = "actix_web")]
impl ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            Error::Io(e) => actix_web::HttpResponse::InternalServerError().json(WhenError {
                type_: ErrorType::Io,
                message: e.to_string(),
                result: "error".to_string(),
            }),
            Error::ReqwestError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::ReqwestError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::MangadexAPIError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::MangadexAPIError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::TokioJoinError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::TokioJoinError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::SerdeJsonError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::SerdeJsonError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::UuidError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError {
                type_: ErrorType::UuidError,
                message: e.to_string(),
                result: "error".to_string(),
            }),
            Error::StringUtf8Error(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::StringUtf8Error,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::StringUTF16Error(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::StringUTF16Error,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::StringParseError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::StringParseError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::Other(e) => actix_web::HttpResponse::InternalServerError().json(WhenError {
                type_: ErrorType::Other,
                message: e.to_string(),
                result: "error".to_string(),
            }),
            Error::MangadexBuilderError(e) => actix_web::HttpResponse::InternalServerError()
                .json(WhenError {
                    type_: ErrorType::ChapterDownloadBuilderError,
                    message: e.to_string(),
                    result: "error".to_string(),
                }),
            Error::ListCoverBuilderError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::ListCoverBuilderError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::DownloadTaskLimitExceded { current, limit } => {
                actix_web::HttpResponse::TooManyRequests().json(WhenError {
                    type_: ErrorType::DownloadTaskLimitExceded,
                    message: format!("Download task limit exceded {current}/{limit}"),
                    result: "error".to_string(),
                })
            }
            Error::TryIntError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::TryIntError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::OneshotRecvError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::OneshotRecvError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::AcquireError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::AcquireError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::RollBacked(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::RollBacked,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
            Error::RwLockError(e) => {
                actix_web::HttpResponse::InternalServerError().json(WhenError {
                    type_: ErrorType::RwLockError,
                    message: e.to_string(),
                    result: "error".to_string(),
                })
            }
        }
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
