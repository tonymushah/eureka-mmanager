use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};

#[get("/manga/{id}/aggregate")]
pub async fn aggregate_manga(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let aggregate = app_state
        .manga_utils()
        .with_id(*id)
        .aggregate_manga_chapters()
        .await?;
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "volumes" : aggregate.volumes
        })
        .to_string(),
    ))
}
