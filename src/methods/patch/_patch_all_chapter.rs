use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use log::info;
use std::path::Path;

/// update all chapters data
#[patch("/chapter/all")]
pub async fn patch_all_chapter(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state.clone());
    let path = app_state.dir_options.chapters_add("");
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<uuid::Uuid> = Vec::new();
        for files in list_dir.flatten() {
            if let Ok(file_type) = files.file_type() {
                if file_type.is_dir() {
                    let id = files
                        .file_name()
                        .to_str()
                        .ok_or(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "can't reconize file".to_string(),
                        ))?
                        .to_string();
                    let util = app_state.chapter_utils().with_id(id.clone());
                    vecs.push(
                        <AppState as AccessChapterUtisWithID>::update(&mut app_state, &util)
                            .await?
                            .data
                            .id,
                    );
                    info!("downloaded chapter data {}", id);
                }
            }
        }
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                    "result" : "ok",
                    "tasks" : "patched",
                    "type" : "collection",
                    "data" : vecs
            })
            .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters directory"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
