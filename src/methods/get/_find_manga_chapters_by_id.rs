use super::DefaultOffsetLimit;
use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_qs::actix::QsQuery;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindMangaChaptersByIdParams {
    #[serde(default = "<FindMangaChaptersByIdParams as DefaultOffsetLimit>::default_offset")]
    pub offset: usize,
    #[serde(default = "<FindMangaChaptersByIdParams as DefaultOffsetLimit>::default_limit")]
    pub limit: usize,
}

impl DefaultOffsetLimit<'_> for FindMangaChaptersByIdParams {
    type OffsetOutput = usize;

    type LimitOutput = usize;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// find all downloaded chapter manga
#[get("/manga/{id}/chapters")]
pub async fn find_manga_chapters_by_id(
    id: web::Path<uuid::Uuid>,
    params: QsQuery<FindMangaChaptersByIdParams>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let to_use = app_state
        .manga_utils()
        .with_id(id.to_string())
        .get_downloaded_chapter(params.offset, params.limit)
        .await?;
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : to_use
        })
        .to_string(),
    ))
}
