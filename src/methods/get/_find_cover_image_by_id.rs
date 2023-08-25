use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use mangadex_api_schema_rust::v5::CoverAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use std::path::Path;

/// find a cover by his id
#[get("/cover/{id}/image")]
pub async fn find_cover_image_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = app_state.dir_options.clone();
    let path = file_dirs.covers_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() {
        let jsons = std::fs::read_to_string(path.as_str())?;
        let cover_data: ApiData<ApiObject<CoverAttributes>> = serde_json::from_str(jsons.as_str())?;
        let filename = cover_data.data.attributes.file_name;
        let filename_path = file_dirs.covers_add(format!("images/{}", filename).as_str());
        Ok(HttpResponse::Ok()
            .content_type(ContentType::jpeg())
            .body(std::fs::read(filename_path)?))
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
