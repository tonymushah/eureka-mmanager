use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};

/// find a chapter (json data) by his id
#[get("/chapter/{id}/data")]
pub async fn find_chapters_data_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let images = app_state
        .chapter_utils()
        .with_id(id.into_inner())
        .get_data_images()?;

    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : images
        })
        .to_string(),
    ))
}
