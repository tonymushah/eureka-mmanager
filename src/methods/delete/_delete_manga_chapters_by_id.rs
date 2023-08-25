use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{delete, web, HttpResponse, Responder};
use mangadex_api_schema_rust::v5::{CoverAttributes, MangaAttributes};
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;

/// delete a  manga
#[delete("/manga/{id}")]
pub async fn delete_manga_chapters_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = app_state.dir_options.clone();
    let path = file_dirs.mangas_add(format!("{}.json", id).as_str());
    let jsons = std::fs::read_to_string(path.as_str())?;
    let manga_data: ApiData<ApiObject<MangaAttributes>> = serde_json::from_str(jsons.as_str())?;
    let cover_id = manga_data
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::CoverArt)
        .ok_or(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            format!("can't find the cover art in manga {}", id),
        ))?
        .id;
    let filename_path1 = file_dirs.covers_add(format!("{}.json", cover_id.hyphenated()).as_str());
    let path2 = file_dirs.covers_add(format!("{}.json", cover_id).as_str());
    let jsons = std::fs::read_to_string(path2.as_str())?;
    let jsons1 = jsons.clone();

    let resp = app_state
        .manga_utils()
        .with_id(id.to_string())
        .find_and_delete_all_downloades()
        .await;
    let jsons = resp?;

    std::fs::remove_file(file_dirs.mangas_add(format!("{}.json", id).as_str()))?;
    std::fs::remove_file(filename_path1)?;
    if let Ok(getted) = serde_json::from_str(jsons1.as_str()) {
        let cover_data: ApiData<ApiObject<CoverAttributes>> = getted;
        let filename = cover_data.data.attributes.file_name;
        let filename_path2 = file_dirs.covers_add(format!("images/{}", filename).as_str());
        std::fs::remove_file(filename_path2)?;
    };

    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : jsons,
            "message" : "deleted "
        })
        .to_string(),
    ))
}
