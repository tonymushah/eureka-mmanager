use super::DefaultOffsetLimit;
use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_qs::actix::QsQuery;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAllDownloadedMangaParams {
    #[serde(default = "<FindAllDownloadedMangaParams as DefaultOffsetLimit>::default_offset")]
    pub offset: usize,
    #[serde(default = "<FindAllDownloadedMangaParams as DefaultOffsetLimit>::default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub title: Option<String>,
}

impl DefaultOffsetLimit<'_> for FindAllDownloadedMangaParams {
    type OffsetOutput = usize;

    type LimitOutput = usize;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// find all downloaded manga
#[get("/manga")]
pub async fn find_all_downloaded_manga(
    params: QsQuery<FindAllDownloadedMangaParams>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let getted = app_state
        .manga_utils()
        .get_downloaded_manga(params.offset, params.limit, params.title.clone())
        .await?;

    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : getted
        })
        .to_string(),
    ))
}
