use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::manga_aggregate::MangaAggregateParams;
use actix_web::{get, web, HttpResponse, Responder};
use serde_qs::actix::QsQuery;

#[get("/manga/{id}/aggregate")]
pub async fn aggregate_manga(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
    params: QsQuery<MangaAggregateParams>,
) -> ManagerCoreResult<impl Responder> {
    let aggregate = app_state
        .manga_utils()
        .with_id(*id)
        .aggregate_manga_chapter(params.into_inner(), app_state.get_ref())
        .await?;
    Ok(HttpResponse::Ok().json(aggregate))
}
