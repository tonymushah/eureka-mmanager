use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use log::info;
use std::path::Path;

/// patch all chapters manga data
#[patch("/chapter/all/patch-manga")]
pub async fn patch_all_chapter_manga(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let path = app_state.dir_options.chapters_add("");
    //info!("{}", path);
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir.flatten() {
            if let Some(id) = files.file_name().to_str() {
                let utils = app_state.chapter_utils().with_id(id.to_string());
                if let Ok(is_there) = utils.is_manga_there() {
                    if !is_there {
                        vecs.push(app_state.patch_manga(&utils).await?);
                        info!("downloaded manga data {}", id.to_string());
                    }
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
