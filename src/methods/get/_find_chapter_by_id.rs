use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::settings::file_history::{HistoryEntry, NoLFAsyncIsIn};
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use log::info;
use std::path::Path;
use std::str::FromStr;

/// find a chapter (json data) by his id
#[get("/chapter/{id}")]
pub async fn find_chapter_by_id(
    id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let app_state : AppState = Into::into(app_state); 
    //let path = format!("chapters/{}/data.json", id);
    let file_dirs = &app_state.dir_options;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data.json", id).as_str());
    if Path::new(path.as_str()).exists() {
        let jsons = std::fs::read_to_string(path.as_str())?;
        let uuid_str = format!("urn:uuid:{}", id);
        match uuid::Uuid::from_str(uuid_str.as_str()) {
            Ok(uuid_data) => {
                if <AppState as NoLFAsyncIsIn<HistoryEntry>>::is_in(&app_state, HistoryEntry::new(uuid_data, mangadex_api_types_rust::RelationshipType::Chapter)).await? {
                    Ok(HttpResponse::Ok()
                        .insert_header(("X-DOWNLOAD-FAILED", "true"))
                        .content_type(ContentType::json())
                        .body(jsons))
                } else {
                    Ok(HttpResponse::Ok()
                        .insert_header(("X-DOWNLOAD-FAILED", "false"))
                        .content_type(ContentType::json())
                        .body(jsons))
                }
            }
            Err(error) => {
                info!("{}", error.to_string());
                Ok(HttpResponse::Ok()
                    .insert_header(("X-DOWNLOAD-FAILED", "false"))
                    .insert_header(("EUREKA-UUID-PARSING-ERROR", "true"))
                    .content_type(ContentType::json())
                    .body(jsons))
            }
        }
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "id" : id.as_str(),
            "message" : "Cannot find the chapter in the api"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
