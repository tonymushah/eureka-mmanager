use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use std::path::Path;

/// find a cover by his id
/// # How to use
/// {app deployed url}/cover/{cover_id}
#[get("/cover/{id}")]
pub async fn find_cover_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = app_state.dir_options.clone();
    let path = file_dirs.covers_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() {
        let jsons = std::fs::read_to_string(path.as_str())?;
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "cover",
            "id" : id.to_string(),
            "message" : "Cannot find the manga in the api"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
