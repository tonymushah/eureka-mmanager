use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use mangadex_api_schema_rust::v5::MangaAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use std::path::Path;

/// find a downloaded manga cover
#[get("/manga/{id}/cover")]
pub async fn find_manga_cover_by_id(
    id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = app_state.dir_options.clone();
    let path = file_dirs.mangas_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() {
        let jsons = std::fs::read_to_string(path.as_str())?;
        let manga_data: ApiData<ApiObject<MangaAttributes>> = serde_json::from_str(jsons.as_str())?;
        let cover_id = manga_data
            .data
            .relationships
            .iter()
            .find(|related| related.type_ == RelationshipType::CoverArt)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("can't find the cover of this manga {}", id),
            ))?
            .id;
        let filename_path =
            file_dirs.covers_add(format!("{}.json", cover_id.hyphenated()).as_str());
        let data = std::fs::read_to_string(filename_path)?;
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(data))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : "Cannot find the manga in the api"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
