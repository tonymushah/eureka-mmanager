use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use uuid::Uuid;

/// find a chapters data-saver (json data) by his id
#[get("/chapter/{id}/data-saver")]
pub async fn find_chapters_data_saver_by_id(
    id: web::Path<Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let images = app_state
        .chapter_utils()
        .with_id(id.into_inner())
        .get_data_saver_images()?;

    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : images
        })
        .to_string(),
    ))
}
