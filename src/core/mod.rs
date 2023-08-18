use actix_web::ResponseError;

#[derive(thiserror::Error, Debug)]
pub enum Error {
    #[error("An std::io::Error captured! \n Details : {0}")]
    Io(#[from] std::io::Error),
    #[error("An internal actix error captured! \n Details : {0}")]
    InternalServerError(#[from] actix_web::Error),
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
    #[error("An error occured when building mangadex_api::utils::download::chapter::ChapterDownload \n Details : {0}")]
    ChapterDownloadBuilderError(#[from] mangadex_api::utils::download::chapter::ChapterDownloadBuilderError),
    #[error("An error occured when building mangadex_api::utils::download::cover::CoverDownload \n Details : {0}")]
    CoverDownloadBuilderError(#[from] mangadex_api::utils::download::cover::CoverDownloadBuilderError),
    #[error("An error occured when building mangadex_api::v5::manga::get::GetManga \n Details : {0}")]
    GetMangaBuilderError(#[from] mangadex_api::v5::manga::get::GetMangaBuilderError),
    #[error("An error occured when building mangadex_api::v5::cover::list::ListCover \n Details : {0}")]
    ListCoverBuilderError(#[from] mangadex_api::v5::cover::list::ListCoverBuilderError)
}

impl ResponseError for Error {
    fn error_response(&self) -> actix_web::HttpResponse<actix_web::body::BoxBody> {
        match self {
            Error::Io(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::InternalServerError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::ReqwestError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::MangadexAPIError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::TokioJoinError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::SerdeJsonError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::UuidError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::StringUtf8Error(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::StringUTF16Error(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::StringParseError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::Other(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::ChapterDownloadBuilderError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::CoverDownloadBuilderError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::GetMangaBuilderError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
            Error::ListCoverBuilderError(e) => actix_web::HttpResponse::InternalServerError().json(WhenError{
                message : e.to_string(),
                result : "error".to_string()
            }),
        }
    }
}

#[derive(serde::Serialize)]
pub struct WhenError{
    message : String,
    result : String
}

pub type ManagerCoreResult<T> = Result<T, Error>; 