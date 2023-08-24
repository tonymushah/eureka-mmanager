use crate::server::AppState;
use crate::settings::files_dirs::DirsOptions;
use crate::{this_api_option, this_api_result};
use actix_web::http::header::ContentType;
use actix_web::{delete, web, HttpResponse, Responder};
use mangadex_api_schema_rust::v5::{CoverAttributes, MangaAttributes};
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use std::path::Path;

// NOTE all delete methods

/// delete a chapter from the api
#[delete("/chapter/{id}")]
pub async fn delete_chapter_by_id(id: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let chapter_path = app_state.dir_options.chapters_add(format!("{}", id).as_str());
    if Path::new(chapter_path.as_str()).exists() {
        this_api_result!(std::fs::remove_dir_all(chapter_path));
        let jsons: serde_json::Value = serde_json::json!({
            "result" : "ok"
        });
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : format!("can't find chapter {}", id)
        });
        HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// delete a  manga
#[delete("/manga/{id}")]
pub async fn delete_manga_chapters_by_id(id: web::Path<String>, app_state: web::Data<AppState>) -> impl Responder {
    let file_dirs = app_state.dir_options.clone();
    let file_dir_clone = file_dirs.clone();
    let path = file_dirs.mangas_add(format!("{}.json", id).as_str());
    let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));
    let manga_data: ApiData<ApiObject<MangaAttributes>> =
        this_api_result!(serde_json::from_str(jsons.as_str()));
    let cover_id = this_api_option!(
        manga_data
            .data
            .relationships
            .iter()
            .find(|related| related.type_ == RelationshipType::CoverArt),
        format!("can't find the cover art in manga {}", id)
    )
    .id;
    let filename_path1 =
        file_dir_clone.covers_add(format!("{}.json", cover_id.hyphenated()).as_str());

    let file_dirs2 = this_api_result!(DirsOptions::new());
    let file_dir_clone2 = file_dirs2.clone();
    let path2 = file_dirs2.covers_add(format!("{}.json", cover_id).as_str());
    let jsons = this_api_result!(std::fs::read_to_string(path2.as_str()));
    let jsons1 = jsons.clone();

    let resp = app_state.manga_utils().with_id(id.to_string()).find_and_delete_all_downloades().await;
    let jsons = this_api_result!(resp);

    this_api_result!(std::fs::remove_file(
        this_api_result!(DirsOptions::new()).mangas_add(format!("{}.json", id).as_str()),
    ));
    this_api_result!(std::fs::remove_file(filename_path1));
    if let Ok(getted) = serde_json::from_str(jsons1.as_str()) {
        let cover_data: ApiData<ApiObject<CoverAttributes>> = getted;
        let filename = cover_data.data.attributes.file_name;
        let filename_path2 = file_dir_clone2.covers_add(format!("images/{}", filename).as_str());
        this_api_result!(std::fs::remove_file(filename_path2));
    };

    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : jsons,
            "message" : "deleted "
        })
        .to_string(),
    )
}
