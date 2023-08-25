use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use itertools::Itertools;
use std::path::Path;

/// find a chapter (json data) by his id
#[get("/chapter/{id}/data")]
pub async fn find_chapters_data_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = &app_state.dir_options;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data", id).as_str());
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let filename_os = files?.file_name().clone();
            let filename = filename_os
                .to_str()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "can't reconize file".to_string().to_string(),
                ))?
                .to_string();
            if !filename.ends_with(".json") {
                vecs.push(filename);
            }
        }
        vecs = vecs.into_iter().unique().collect();
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : vecs
            })
            .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.to_string(),
            "message" : format!("Cannot find the chapter {} data in the api", id)
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
