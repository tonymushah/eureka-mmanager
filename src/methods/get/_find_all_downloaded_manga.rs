#[cfg(feature = "actix_web")]
use crate::core::ManagerCoreResult;
#[cfg(feature = "actix_web")]
use crate::server::AppState;
#[cfg(feature = "actix_web")]
use actix_web::{get, web, HttpResponse, Responder};
#[cfg(feature = "actix_web")]
use serde_qs::actix::QsQuery;
#[cfg(feature = "actix_web")]
use mangadex_api_input_types::manga::list::MangaListParams;

/// find all downloaded manga

#[cfg(feature = "actix_web")]
#[get("/manga")]
pub async fn find_all_downloaded_manga(
    params: QsQuery<MangaListParams>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let getted = app_state
        .manga_utils()
        .get_downloaded_manga(params.into_inner())
        .await?;

    Ok(HttpResponse::Ok().json(getted))
}
