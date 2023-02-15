use crate::chapter_download::{download_chapter, download_chapter_saver};
use crate::cover_download::{
    all_covers_download_quality_by_manga_id, cover_download_by_cover, cover_download_by_manga_id,
    cover_download_quality_by_cover, cover_download_quality_by_manga_id,
};
use crate::manga_download::download_manga;
use crate::settings::file_history::{init_history_dir, load_history};
use crate::settings::{
    initialise_data_dir, initialise_settings_dir, verify_data_dir, verify_settings_dir,
};
use crate::utils::{
    find_and_delete_all_downloades_by_manga_id,
    get_query_hash_value_or_else, is_chapter_manga_there, patch_manga_by_chapter,
};
use actix_web::dev::{self, Server, ServiceResponse};
use actix_web::http::header::{self, ContentType};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{
    delete, get, http::StatusCode, patch, put, web, App, HttpRequest, HttpResponse, HttpServer,
    Responder,
};
use log::{info, warn};
use mangadex_api_schema::v5::{CoverAttributes, MangaAttributes};
use mangadex_api_schema::{ApiData, ApiObject};
use mangadex_api_types::RelationshipType;
use settings::files_dirs::DirsOptions;
use settings::server_options;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use std::num::ParseIntError;
use std::path::Path;
use utils::{get_downloaded_manga, query_string_to_hash_map, get_downloaded_chapter_of_a_manga, get_all_downloaded_chapters, get_downloaded_cover_of_a_manga_collection};

pub mod chapter_download;
pub mod cover_download;
pub mod manga_download;
pub mod settings;
pub mod utils;
// NOTE all get methods

fn get_params(request: HttpRequest) -> HashMap<String, String> {
    return match query_string_to_hash_map(request.query_string()) {
        Ok(value) => value,
        Err(error) => {
            println!("{}", error.to_string());
            HashMap::new()
        }
    };
}

/// try if the app is ok
/// # How to use
/// {app deployed url}/
#[get("/")]
async fn hello(/*request: HttpRequest*/) -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "message" : "The mangadex desktop api works !!"
        })
        .to_string(),
    )
}

#[macro_export]
macro_rules! this_api_result {
    ($to_use:expr) => {
        match $to_use {
            Err(e) => {
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", e.to_string())
                });
                return HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body(jsons.to_string());
            }
            Ok(f) => f,
        }
    };
}

#[macro_export]
macro_rules! this_api_option {
    ($to_use:expr, $message:expr) => {
        match $to_use {
            Some(d) => d,
            None => {
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : $message
                });
                return HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body(jsons.to_string());
            }
        }
    };
}

/// Find a downloaded manga
/// # How to use
/// {app deployed url}/manga/{manga_id}
#[get("/manga/{id}")]
async fn find_manga_by_id(id: web::Path<String>) -> impl Responder {
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
async fn find_cover_by_id(id: web::Path<String>) -> impl Responder {
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
async fn find_cover_image_by_id(id: web::Path<String>) -> impl Responder {
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
async fn find_manga_cover_by_id(id: web::Path<String>) -> impl Responder {
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
async fn find_manga_covers_by_id(id: web::Path<String>, request : HttpRequest) -> impl Responder {
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
async fn find_chapters_data_by_id(id: web::Path<String>) -> impl Responder {
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
async fn find_chapters_data_saver_by_id(id: web::Path<String>) -> impl Responder {
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
async fn find_chapters_data_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
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
async fn find_chapters_data_saver_img_by_id(data: web::Path<(String, String)>) -> impl Responder {
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
async fn find_chapter_by_id(id: web::Path<String>) -> impl Responder {
    //let path = format!("chapters/{}/data.json", id);
    let file_dirs = this_api_result!(DirsOptions::new());
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data.json", id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons.to_string())
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

/// get all dowloaded chapter
#[get("/chapter")]
async fn find_all_downloaded_chapter(request : HttpRequest) -> impl Responder {
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
async fn find_all_downloaded_manga(request: HttpRequest) -> impl Responder {
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
async fn find_manga_chapters_by_id(id: web::Path<String>, request: HttpRequest) -> impl Responder {
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

// NOTE update api data

/// update a cover json data by his id
#[patch("/cover/{id}")]
async fn update_cover_by_id(id: web::Path<String>) -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).covers_add(format!("{}.json", id).as_str());
    let http_client = reqwest::Client::new();
    let get_cover = this_api_result!(http_client
                .get(
                    format!("{}/cover/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user",
                        mangadex_api::constants::API_URL,
                        id
                    )
                )
                .send()
                .await
            );

    let bytes_ = this_api_result!(get_cover.bytes().await);

    let mut cover_data = this_api_result!(File::create(path.clone()));

    this_api_result!(cover_data.write_all(&bytes_));

    let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(jsons.to_string())
}

/// update a chapter by his id
#[patch("/chapter/{id}")]
async fn update_chapter_by_id(id: web::Path<String>) -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).chapters_add(format!("{}.json", id).as_str());
    let http_client = reqwest::Client::new();
    let get_cover = this_api_result!(http_client
                .get(
                    format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user",
                        mangadex_api::constants::API_URL,
                        id
                    )
                )
                .send()
                .await
            );

    let bytes_ = this_api_result!(get_cover.bytes().await);

    let mut cover_data = this_api_result!(File::create(path.clone()));

    this_api_result!(cover_data.write_all(&bytes_));

    let jsons = this_api_result!(std::fs::read_to_string(path.as_str()));

    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(jsons.to_string())
}

/// update all chapters data
#[patch("/chapter/all")]
async fn patch_all_chapter() -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).chapters_add("");
    if Path::new(path.as_str()).exists() == true {
        let list_dir = this_api_result!(std::fs::read_dir(path.as_str()));
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir {
            let id = this_api_option!(
                this_api_result!(files).file_name().to_str(),
                format!("can't reconize file")
            )
            .to_string();
            vecs.push(this_api_result!(utils::update_chap_by_id(id.clone()).await));
            info!("downloaded chapter data {}", id);
        }
        HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                    "result" : "ok",
                    "tasks" : "patched",
                    "type" : "collection",
                    "data" : vecs
            })
            .to_string(),
        )
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters directory"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// patch all chapters manga data
#[patch("/chapter/all/patch-manga")]
async fn patch_all_chapter_manga() -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).chapters_add("");
    //info!("{}", path);
    if Path::new(path.as_str()).exists() == true {
        let list_dir = this_api_result!(std::fs::read_dir(path.as_str()));
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir {
            let id = this_api_option!(
                this_api_result!(files).file_name().to_str(),
                format!("can't reconize file")
            )
            .to_string();
            let id_clone = id.clone();
            let id_clone_clone = id.clone();
            if this_api_result!(is_chapter_manga_there(id)) == false {
                vecs.push(this_api_result!(
                    utils::patch_manga_by_chapter(id_clone).await
                ));
            }

            info!("downloaded manga data {}", id_clone_clone);
        }
        HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                "result" : "ok",
                "tasks" : "patched",
                "type" : "collection",
                "data" : vecs
            })
            .to_string(),
        )
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters directory"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// patch a chapter manga data
#[patch("/chapter/{id}/patch-manga")]
async fn update_chapter_manga_by_id(id: web::Path<String>) -> impl Responder {
    //let path = format!("chapters/{}/data.json", id);
    let path = this_api_result!(DirsOptions::new())
        .chapters_add(format!("chapters/{}/data.json", id).as_str());

    if Path::new(path.as_str()).exists() == true {
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(this_api_result!(patch_manga_by_chapter(id.to_string()).await).to_string())
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters data"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

/// patch all manga cover
#[patch("/manga/all/cover")]
async fn patch_all_manga_cover() -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).mangas_add("");
    if Path::new(path.as_str()).exists() == true {
        let list_dir = this_api_result!(std::fs::read_dir(path.as_str()));
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir {
            let manga_id = this_api_option!(
                this_api_result!(files).file_name().to_str(),
                format!("can't reconize file")
            )
            .to_string()
            .replace(".json", "");
            //let mg_id = manga_id.clone();
            vecs.push(this_api_result!(
                cover_download_by_manga_id(manga_id.as_str()).await
            ));
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
        "message" : "can't find the chapters directory"
        });
        HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string())
    }
}

// NOTE all delete methods

/// delete a chapter from the api
#[delete("/chapter/{id}")]
async fn delete_chapter_by_id(id: web::Path<String>) -> impl Responder {
    let jsons: serde_json::Value;
    let chapter_path = this_api_result!(DirsOptions::new()).mangas_add(format!("{}", id).as_str());
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
async fn delete_manga_chapters_by_id(id: web::Path<String>) -> impl Responder {
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

// NOTE All download methods

/// download a manga (req only)
#[put("/manga/{id}")]
async fn download_manga_by_id(id: web::Path<String>) -> impl Responder {
    let http_client = reqwest::Client::new();
    let manga_urn_uuid = format!("{}", id);
    let manga_id = this_api_result!(uuid::Uuid::parse_str(manga_urn_uuid.as_str()));
    let download_manga_query = download_manga(http_client, manga_id).await;
    this_api_result!(download_manga_query);

    let jsons = serde_json::json!({
        "result" : "ok",
        "type" : "manga",
        "id" : id.as_str()
    });
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(jsons.to_string())
}

/// download all manga covers
#[put("/manga/{id}/covers")]
async fn download_manga_covers(id: web::Path<String>) -> impl Responder {
    let response = this_api_result!(
        all_covers_download_quality_by_manga_id(format!("{id}").as_str(), 100).await
    );
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download the top manga cover
#[put("/manga/{id}/cover")]
async fn download_manga_cover(id: web::Path<String>) -> impl Responder {
    let response = this_api_result!(cover_download_by_manga_id(format!("{id}").as_str()).await);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download the top manga cover with defined quality
#[put("/manga/{id}/cover/{quality}")]
async fn download_manga_cover_quality(path_var: web::Path<(String, u32)>) -> impl Responder {
    let id = (*(path_var.0)).to_string();
    let quality = path_var.1;
    let response = this_api_result!(
        cover_download_quality_by_manga_id(format!("{id}").as_str(), quality).await
    );
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download cover by id
#[put("/cover/{id}")]
async fn download_cover(id: web::Path<String>) -> impl Responder {
    let response = this_api_result!(cover_download_by_cover(format!("{id}").as_str()).await);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download cover by id with defined quality
#[put("/cover/{id}/{quality}")]
async fn download_cover_quality(path_var: web::Path<(String, u32)>) -> impl Responder {
    let id = (*(path_var.0)).to_string();
    let quality = path_var.1;
    let response =
        this_api_result!(cover_download_quality_by_cover(format!("{id}").as_str(), quality).await);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download chapter by id
#[put("/chapter/{id}")]
async fn download_chapter_byid(id: web::Path<String>) -> impl Responder {
    info!("Start downloading chapter {id}");

    let response = this_api_result!(download_chapter(format!("{id}").as_str()).await);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download chapter data by id
#[put("/chapter/{id}/data")]
async fn download_chapter_data_byid(id: web::Path<String>) -> impl Responder {
    format!("Start downloading manga {id}");
    let response = this_api_result!(download_chapter(format!("{id}").as_str()).await);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download chapter data-saver by id
#[put("/chapter/{id}/data-saver")]
async fn download_chapter_data_saver_byid(id: web::Path<String>) -> impl Responder {
    format!("Start downloading manga {id}");
    let response = this_api_result!(download_chapter_saver(format!("{id}").as_str()).await);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// url not found handler
///
///

fn not_found_message<B>(
    mut res: dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    let (req, res) = res.into_parts();
    let json = serde_json::json!({
        "result" : "error",
        "message" : format!("Ressource {} not found", req.path())
    });
    let res = res.set_body(json.to_string());
    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();
    Ok(ErrorHandlerResponse::Response(res))
}

pub fn launch_async_server(address: &str, port: u16) -> std::io::Result<Server> {
    Ok(HttpServer::new(|| {
        App::new()
            .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found_message))
            /*
                get Methods
            */
            .service(find_manga_by_id)
            .service(find_cover_by_id)
            .service(find_cover_image_by_id)
            .service(find_manga_cover_by_id)
            .service(find_manga_covers_by_id)
            .service(find_chapters_data_by_id)
            .service(find_chapters_data_saver_by_id)
            .service(find_chapters_data_img_by_id)
            .service(find_chapters_data_saver_img_by_id)
            .service(find_chapter_by_id)
            .service(find_all_downloaded_chapter)
            .service(find_all_downloaded_manga)
            .service(find_manga_chapters_by_id)
            .service(hello)
            /*
                patch methods
            */
            .service(update_cover_by_id)
            .service(update_chapter_by_id)
            .service(patch_all_chapter)
            .service(patch_all_chapter_manga)
            .service(update_chapter_manga_by_id)
            .service(patch_all_manga_cover)
            /*
                delete methods
            */
            .service(delete_chapter_by_id)
            .service(delete_manga_chapters_by_id)
            /*
                put methods
            */
            .service(download_manga_by_id)
            .service(download_manga_covers)
            .service(download_manga_cover)
            .service(download_manga_cover_quality)
            .service(download_cover)
            .service(download_cover_quality)
            .service(download_chapter_byid)
            .service(download_chapter_data_byid)
            .service(download_chapter_data_saver_byid)
    })
    .bind((address, port))?
    .run())
}

#[actix_web::main]
/// it's launch the server in the given adrress and the given port
/// a call like this
///
/// # Example
/// ```
/// fn main() -> std::io:Result<()> {
///     let address = "127.0.0.1";
///     let port : u16 = 8090;
///     launch_server(address, port)
///     // it launch the server at 127.0.0.1:8090
/// }
/// ```
pub async fn launch_server(address: &str, port: u16) -> std::io::Result<()> {
    info!("launching mangadex-desktop-api on {}:{}", address, port);
    let habdle = launch_async_server(address, port)?.await;
    info!("closing mangadex-desktop-api on {}:{}", address, port);
    habdle
}

pub fn launch_async_server_default() -> std::io::Result<Server> {
    info!("launching server");
    let serve: server_options::ServerOptions = match server_options::ServerOptions::new() {
        Ok(data) => data,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };
    launch_async_server(serve.hostname.as_str(), serve.port)
}

/// Verify if the data dir and the settings are all there
/// if on of them are not defined or not found , it automatically create the dir corresponding to the error
pub fn verify_all_fs() -> std::io::Result<()> {
    match verify_settings_dir() {
        Ok(data) => {
            info!("{}", data);
        }
        Err(error) => {
            warn!("{}", error);
            warn!("Settings dir not found ");
            info!("Initializing...");
            match initialise_settings_dir() {
                Ok(data) => data,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ));
                }
            };
        }
    };
    info!("Initilized settings dir !");
    match verify_data_dir() {
        Ok(data) => {
            info!("{}", data);
        }
        Err(error) => {
            warn!("{}", error);
            warn!("Data dir not found \n");
            info!("\tInitializing...");
            match initialise_data_dir() {
                Ok(data) => data,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ));
                }
            };
        }
    }
    load_history()?;
    Ok(())
}

/// It's launch the server with the given data in the settings/server_option.json
///
/// # Example

/// if we have a settings/server_option.json like this :
/// ```
/// {
///   "hostname" : "127.0.0.1",
///    "port" : 8090
/// }
/// ```
///
/// and we launch the function :
/// ```
/// fn main() -> std::io::Result<()> {
///     launch_server_default()
///     // it will launch the server at 127.0.0.1:8090
/// }
/// ```
pub fn launch_server_default() -> std::io::Result<()> {
    info!("launching server");
    let serve: server_options::ServerOptions = match server_options::ServerOptions::new() {
        Ok(data) => data,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };
    launch_server(serve.hostname.as_str(), serve.port)
}
