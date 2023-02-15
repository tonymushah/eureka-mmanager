use crate::methods::get_params;
use crate::settings::get_history;
use crate::{this_api_result, this_api_option};
use crate::utils::{
    get_query_hash_value_or_else
};
use actix_web::http::header::{ContentType};
use actix_web::{
    get, web, HttpRequest, HttpResponse,
    Responder,
};
use log::info;
use mangadex_api_schema::v5::{CoverAttributes, MangaAttributes};
use mangadex_api_schema::{ApiData, ApiObject};
use mangadex_api_types::RelationshipType;
use crate::settings::files_dirs::DirsOptions;
use std::num::ParseIntError;
use std::path::Path;
use std::str::{FromStr, ParseBoolError};
use crate::utils::{get_downloaded_manga, get_downloaded_chapter_of_a_manga, get_all_downloaded_chapters, get_downloaded_cover_of_a_manga_collection};

/// try if the app is ok
/// # How to use
/// {app deployed url}/
#[get("/")]
pub async fn hello(/*request: HttpRequest*/) -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "message" : "The mangadex desktop api works !!"
        })
        .to_string(),
    )
}
/// Find a downloaded manga
/// # How to use
/// {app deployed url}/manga/{manga_id}
#[get("/manga/{id}")]
pub async fn find_manga_by_id(id: web::Path<String>) -> impl Responder {
    let file_dirs = this_api_result!(DirsOptions::new());
    let path = file_dirs.mangas_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));
        return HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons.to_string());
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : "Cannot find the manga in the api"
        });
        return HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string());
    }
}
/// find a cover by his id
/// # How to use
/// {app deployed url}/cover/{cover_id}
#[get("/cover/{id}")]
pub async fn find_cover_by_id(id: web::Path<String>) -> impl Responder {
    let file_dirs = this_api_result!(DirsOptions::new());
    let path = file_dirs.covers_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "cover",
            "id" : id.as_str(),
            "message" : "Cannot find the manga in the api"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// find a cover by his id
#[get("/cover/{id}/image")]
pub async fn find_cover_image_by_id(id: web::Path<String>) -> impl Responder {
    let file_dirs = this_api_result!(DirsOptions::new());
    let file_dir_clone = file_dirs.clone();
    let path = file_dirs.covers_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));
        let cover_data: ApiData<ApiObject<CoverAttributes>> =
            this_api_result!(serde_json::from_str(jsons.as_str()));
        let filename = cover_data.data.attributes.file_name;
        let filename_path = file_dir_clone.covers_add(format!("images/{}", filename).as_str());
        HttpResponse::Ok()
            .content_type(ContentType::jpeg())
            .body(this_api_result!(std::fs::read(filename_path)))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "cover",
            "id" : id.as_str(),
            "message" : "Cannot find the manga in the api"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// find a downloaded manga cover
#[get("/manga/{id}/cover")]
pub async fn find_manga_cover_by_id(id: web::Path<String>) -> impl Responder {
    let file_dirs = this_api_result!(DirsOptions::new());
    let file_dir_clone = file_dirs.clone();
    let path = file_dirs.mangas_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));
        let manga_data: ApiData<ApiObject<MangaAttributes>> =
            this_api_result!(serde_json::from_str(jsons.as_str()));
        let cover_id = this_api_option!(
            manga_data
                .data
                .relationships
                .iter()
                .find(|related| related.type_ == RelationshipType::CoverArt),
            format!("can't find the cover of this manga {}", id)
        )
        .id;
        let filename_path =
            file_dir_clone.covers_add(format!("{}.json", cover_id.hyphenated()).as_str());
        let data = this_api_result!(std::fs::read_to_string(filename_path));
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(data)
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : "Cannot find the manga in the api"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// find a downloaded covers manga
#[get("/manga/{id}/covers")]
pub async fn find_manga_covers_by_id(id: web::Path<String>, request : HttpRequest) -> impl Responder {
    let query = get_params(request);
    let offset: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "offset".to_string(), "0".to_string())
            .as_str()
            .parse();
    let offset = this_api_result!(offset);
    let limit: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "limit".to_string(), "10".to_string())
            .as_str()
            .parse();
    let limit = this_api_result!(limit);
    let getted = this_api_result!(get_downloaded_cover_of_a_manga_collection(format!("{}", id), offset, limit));
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : getted
            })
        .to_string(),
    )
}

/// find a chapter (json data) by his id
#[get("/chapter/{id}/data")]
pub async fn find_chapters_data_by_id(id: web::Path<String>) -> impl Responder {
    let file_dirs = this_api_result!(DirsOptions::new());
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let list_dir = this_api_result!(std::fs::read_dir(path.as_str()));
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let filename_os = this_api_result!(files).file_name().clone();
            let filename =
                this_api_option!(filename_os.to_str(), format!("can't reconize file")).to_string();
            if filename.ends_with(".json") == false {
                vecs.push(filename);
            }
        }
        HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : vecs
            })
            .to_string(),
        )
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : format!("Cannot find the chapter {} data in the api", id)
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// find a chapters data-saver (json data) by his id
#[get("/chapter/{id}/data-saver")]
pub async fn find_chapters_data_saver_by_id(id: web::Path<String>) -> impl Responder {
    let file_dirs = this_api_result!(DirsOptions::new());
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data-saver", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let list_dir = this_api_result!(std::fs::read_dir(path.as_str()));
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let filename_os = this_api_result!(files).file_name().clone();
            let filename =
                this_api_option!(filename_os.to_str(), format!("can't reconize file")).to_string();
            if filename.ends_with(".json") == false {
                vecs.push(filename);
            }
        }
        HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : vecs
            })
            .to_string(),
        )
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : format!("Cannot find the chapter {} data-saver in the api", id)
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}
/// find a chapters data image by his id
#[get("/chapter/{id}/data/{filename}")]
pub async fn find_chapters_data_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
    let (id, filename) = data.into_inner();
    let file_dirs = this_api_result!(DirsOptions::new());
    let path = file_dirs.chapters_add(format!("{}/data/{}", id, filename).as_str());
    if Path::new(path.as_str()).exists() == true {
        HttpResponse::Ok()
            .content_type(ContentType::jpeg())
            .body(this_api_result!(std::fs::read(path)))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : format!("Cannot find the chapter {} data-saver in the api", id)
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// find a chapters data-saver image by his id
#[get("/chapter/{id}/data-saver/{filename}")]
pub async fn find_chapters_data_saver_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
    let (id, filename) = data.into_inner();
    let file_dirs = this_api_result!(DirsOptions::new());
    let path = file_dirs.chapters_add(format!("{}/data-saver/{}", id, filename).as_str());
    if Path::new(path.as_str()).exists() == true {
        HttpResponse::Ok()
            .content_type(ContentType::jpeg())
            .body(this_api_result!(std::fs::read(path)))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : format!("Cannot find the chapter {} data-saver in the api", id)
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// find a chapter (json data) by his id
#[get("/chapter/{id}")]
pub async fn find_chapter_by_id(id: web::Path<String>) -> impl Responder {
    //let path = format!("chapters/{}/data.json", id);
    let file_dirs = this_api_result!(DirsOptions::new());
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data.json", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        
        let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));
        let history = this_api_result!(get_history());
        match history.get_mut(&RelationshipType::Chapter) {
            None => {
                HttpResponse::Ok()
                    .insert_header(("X-DOWNLOAD-FAILED", "false"))
                    .content_type(ContentType::json())
                    .body(jsons.to_string())
            }, 
            Some(history_) => {
                let uuid_str = format!("urn:uuid:{}", id);
                match uuid::Uuid::from_str(uuid_str.as_str()) {
                    Ok(uuid_data) => {
                        if history_.get_history().is_in(uuid_data) == true {
                            HttpResponse::Ok()
                                .insert_header(("X-DOWNLOAD-FAILED", "true"))
                                .content_type(ContentType::json())
                                .body(jsons.to_string())
                        } else {
                            HttpResponse::Ok()
                                .insert_header(("X-DOWNLOAD-FAILED", "false"))
                                .content_type(ContentType::json())
                                .body(jsons.to_string())
                        }
                    },
                    Err(error) => {
                        info!("{}", error.to_string());
                        HttpResponse::Ok()
                            .insert_header(("X-DOWNLOAD-FAILED", "false"))
                            .insert_header(("EUREKA-UUID-PARSING-ERROR", "true"))
                            .content_type(ContentType::json())
                            .body(jsons.to_string())
                    }
                }
                
            }
        }
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "id" : id.as_str(),
            "message" : "Cannot find the chapter in the api"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// get all dowloaded chapter
#[get("/chapter")]
pub async fn find_all_downloaded_chapter(request : HttpRequest) -> impl Responder {
    let query = get_params(request);
    let offset: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "offset".to_string(), "0".to_string())
            .as_str()
            .parse();
    let offset = this_api_result!(offset);
    let limit: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "limit".to_string(), "10".to_string())
            .as_str()
            .parse();
    let limit = this_api_result!(limit);
    //let include_failed : Result<boolean, ParseBoolError> = 
    let getted = this_api_result!(get_all_downloaded_chapters(offset, limit));
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : getted
            })
        .to_string(),
    )
}

/// find all downloaded manga
#[get("/manga")]
pub async fn find_all_downloaded_manga(request: HttpRequest) -> impl Responder {
    let query = get_params(request);
    let offset: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "offset".to_string(), "0".to_string())
            .as_str()
            .parse();
    let offset = this_api_result!(offset);
    let limit: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "limit".to_string(), "10".to_string())
            .as_str()
            .parse();
    let limit = this_api_result!(limit);

    let getted = this_api_result!(get_downloaded_manga(offset, limit));
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : getted
            })
        .to_string(),
    )
}

/// find all downloaded chapter manga
#[get("/manga/{id}/chapters")]
pub async fn find_manga_chapters_by_id(id: web::Path<String>, request: HttpRequest) -> impl Responder {
    let query = get_params(request);
    let offset: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "offset".to_string(), "0".to_string())
            .as_str()
            .parse();
    let offset = this_api_result!(offset);
    let limit: Result<usize, ParseIntError> =
        get_query_hash_value_or_else(&query, "limit".to_string(), "10".to_string())
            .as_str()
            .parse();
    let limit = this_api_result!(limit);
    let to_use = this_api_result!(get_downloaded_chapter_of_a_manga(id.to_string(), offset, limit).await);
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : to_use
        })
        .to_string(),
    )
}
