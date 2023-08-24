use crate::server::AppState;
use crate::{this_api_result, this_api_option};
use actix_web::http::header::ContentType;
use actix_web::{
    patch, web, HttpResponse,
    Responder,
};
use log::info;
use crate::settings::files_dirs::DirsOptions;
use std::fs::File;
use std::io::Write;
use std::path::Path;

// NOTE update api data

/// update a cover json data by his id
#[patch("/cover/{id}")]
pub async fn update_cover_by_id(id: web::Path<String>) -> impl Responder {
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
        .body(jsons)
}

/// update a chapter by his id
#[patch("/chapter/{id}")]
pub async fn update_chapter_by_id(id: web::Path<String>) -> impl Responder {
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
        .body(jsons)
}

/// update all chapters data
#[patch("/chapter/all")]
pub async fn patch_all_chapter(data: web::Data<AppState>) -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).chapters_add("");
    if Path::new(path.as_str()).exists() {
        let list_dir = this_api_result!(std::fs::read_dir(path.as_str()));
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir {
            let id = this_api_option!(
                this_api_result!(files).file_name().to_str(),
                format!("can't reconize file")
            )
            .to_string();
            vecs.push(this_api_result!(update_chap_by_id(id.clone(), data.http_client.clone()).await));
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
pub async fn patch_all_chapter_manga(data: web::Data<AppState>) -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).chapters_add("");
    //info!("{}", path);
    if Path::new(path.as_str()).exists() {
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
            if !this_api_result!(is_chapter_manga_there(id)) {
                vecs.push(this_api_result!(
                    patch_manga_by_chapter(id_clone, data.http_client.clone()).await
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
pub async fn update_chapter_manga_by_id(id: web::Path<String>, data: web::Data<AppState>) -> impl Responder {
    //let path = format!("chapters/{}/data.json", id);
    let path = this_api_result!(DirsOptions::new())
        .chapters_add(format!("chapters/{}/data.json", id).as_str());

    if Path::new(path.as_str()).exists() {
        HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(this_api_result!(patch_manga_by_chapter(id.to_string(), data.http_client.clone()).await).to_string())
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
pub async fn patch_all_manga_cover(data: web::Data<AppState>) -> impl Responder {
    let path = this_api_result!(DirsOptions::new()).mangas_add("");
    if Path::new(path.as_str()).exists() {
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
                cover_download_by_manga_id(manga_id.as_str(), data.http_client.clone()).await
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
