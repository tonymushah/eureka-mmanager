use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::ExtractData;
use actix_web::http::header::ContentType;
use actix_web::{delete, web, HttpResponse, Responder};

// NOTE all delete methods

/// delete a chapter from the api
#[delete("/chapter/{id}")]
pub async fn delete_chapter_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    app_state.chapter_utils().with_id(*id).delete()?;
    let jsons: serde_json::Value = serde_json::json!({
        "result" : "ok"
    });
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(jsons.to_string()))
}
