use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use std::path::Path;

/// find a chapters data-saver image by his id
#[get("/chapter/{id}/data-saver/{filename}")]
pub async fn find_chapters_data_saver_img_by_id(
    data: web::Path<(String, String)>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let (id, filename) = data.into_inner();
    let file_dirs = &app_state.dir_options;
    let path = file_dirs.chapters_add(format!("{}/data-saver/{}", id, filename).as_str());
    if Path::new(path.as_str()).exists() {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::jpeg())
            .body(std::fs::read(path)?))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : format!("Cannot find the chapter {} data-saver in the api", id)
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
