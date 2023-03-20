use crate::{this_api_option, this_api_result};
use crate::utils::{
    find_and_delete_all_downloades_by_manga_id
};
use actix_web::http::header::{ContentType};
use actix_web::{
    delete, web, HttpResponse,
    Responder,
};
use mangadex_api_schema::v5::{CoverAttributes, MangaAttributes};
use mangadex_api_schema::{ApiData, ApiObject};
use mangadex_api_types::RelationshipType;
use crate::settings::files_dirs::DirsOptions;
use std::path::Path;

// NOTE all delete methods

/// delete a chapter from the api
#[delete("/chapter/{id}")]
pub async fn delete_chapter_by_id(id: web::Path<String>) -> impl Responder {
    let jsons: serde_json::Value;
    let chapter_path = this_api_result!(DirsOptions::new()).chapters_add(format!("{}", id).as_str());
    if Path::new(chapter_path.as_str()).exists() == true {
        this_api_result!(std::fs::remove_dir_all(chapter_path));
        jsons = serde_json::json!({
            "result" : "ok"
        });
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : format!("can't find chapter {}", id)
        });
        return HttpResponse::InternalServerError()
            .content_type(ContentType::json())
            .body(jsons.to_string());
    }

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(jsons.to_string())
}

/// delete a  manga
#[delete("/manga/{id}")]
pub async fn delete_manga_chapters_by_id(id: web::Path<String>) -> impl Responder {
    let file_dirs = this_api_result!(DirsOptions::new());
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

    let resp = find_and_delete_all_downloades_by_manga_id(id.to_string()).await;
    let jsons = this_api_result!(resp);

    this_api_result!(std::fs::remove_file(
        this_api_result!(DirsOptions::new()).mangas_add(format!("{}.json", id).as_str()),
    ));
    this_api_result!(std::fs::remove_file(filename_path1));
    match serde_json::from_str(jsons1.as_str()) {
        Ok(getted) => {
            let cover_data: ApiData<ApiObject<CoverAttributes>> = getted;
            let filename = cover_data.data.attributes.file_name;
            let filename_path2 =
                file_dir_clone2.covers_add(format!("images/{}", filename).as_str());
            this_api_result!(std::fs::remove_file(filename_path2));
        }
        Err(_) => {}
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
