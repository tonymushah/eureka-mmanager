use super::DefaultOffsetLimit;
use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_qs::actix::QsQuery;

#[derive(Debug, Serialize, Deserialize)]
pub struct FindMangaCoversByIdParams {
    #[serde(default = "<FindMangaCoversByIdParams as DefaultOffsetLimit>::default_offset")]
    pub offset: usize,
    #[serde(default = "<FindMangaCoversByIdParams as DefaultOffsetLimit>::default_limit")]
    pub limit: usize,
}

impl Default for FindMangaCoversByIdParams {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 10,
        }
    }
}

impl DefaultOffsetLimit<'_> for FindMangaCoversByIdParams {
    type OffsetOutput = usize;

    type LimitOutput = usize;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// find a downloaded covers manga
#[get("/manga/{id}/covers")]
pub async fn find_manga_covers_by_id(
    id: web::Path<String>,
    params: QsQuery<FindMangaCoversByIdParams>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let offset = params.offset;
    let limit = params.limit;
    let utils = app_state.manga_utils().with_id(format!("{}", id));
    let getted = utils
        .get_downloaded_cover_of_a_manga_collection(offset, limit, &mut app_state)
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
