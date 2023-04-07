use crate::download::chapter::{download_chapter, download_chapter_saver};
use crate::download::cover::{
    all_covers_download_quality_by_manga_id, cover_download_by_cover, cover_download_by_manga_id,
    cover_download_quality_by_cover, cover_download_quality_by_manga_id,
};
use crate::download::manga::download_manga;
use crate::this_api_result;
use actix_web::http::header::{ContentType};
use actix_web::{
    put, web, HttpResponse,
    Responder,
};
use log::{info};

// NOTE All download methods

/// download a manga (req only)
#[put("/manga/{id}")]
pub async fn download_manga_by_id(id: web::Path<String>) -> impl Responder {
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
pub async fn download_manga_covers(id: web::Path<String>) -> impl Responder {
    let response = this_api_result!(
        all_covers_download_quality_by_manga_id(format!("{id}").as_str(), 100).await
    );
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download the top manga cover
#[put("/manga/{id}/cover")]
pub async fn download_manga_cover(id: web::Path<String>) -> impl Responder {
    let response = this_api_result!(cover_download_by_manga_id(format!("{id}").as_str()).await);
    HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string())
}

/// download the top manga cover with defined quality
#[put("/manga/{id}/cover/{quality}")]
pub async fn download_manga_cover_quality(path_var: web::Path<(String, u32)>) -> impl Responder {
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
pub async fn download_cover(id: web::Path<String>) -> impl Responder {
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
