use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{delete, web, HttpResponse, Responder};
use std::path::Path;

// NOTE all delete methods

/// delete a chapter from the api
#[delete("/chapter/{id}")]
pub async fn delete_chapter_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let chapter_path = app_state
        .dir_options
        .chapters_add(format!("{}", id).as_str());
    if Path::new(chapter_path.as_str()).exists() {
        std::fs::remove_dir_all(chapter_path)?;
        let jsons: serde_json::Value = serde_json::json!({
            "result" : "ok"
        });
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : format!("can't find chapter {}", id)
        });
        Ok(HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}