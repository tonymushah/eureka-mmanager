use super::DefaultOffsetLimit;
use crate::core::{Error, ManagerCoreResult};
use crate::server::traits::AccessHistory;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use itertools::Itertools;
use log::info;
use mangadex_api_schema_rust::v5::{CoverAttributes, MangaAttributes};
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use serde::{Deserialize, Serialize};
use serde_qs::actix::QsQuery;
use std::path::Path;
use std::str::FromStr;

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
pub async fn find_manga_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = app_state.dir_options.clone();
    let path = file_dirs.mangas_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() {
        let jsons = std::fs::read_to_string(path.as_str())?;
        return Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons));
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.to_string(),
            "message" : "Cannot find the manga in the api"
        });
        return Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()));
    }
}
/// find a cover by his id
/// # How to use
/// {app deployed url}/cover/{cover_id}
#[get("/cover/{id}")]
pub async fn find_cover_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = app_state.dir_options.clone();
    let path = file_dirs.covers_add(format!("{}.json", id).as_str());
    if Path::new(path.as_str()).exists() {
        let jsons = std::fs::read_to_string(path.as_str())?;
        Ok(HttpResponse::Ok()
            .content_type(ContentType::json())
            .body(jsons))
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

#[derive(Debug, Serialize, Deserialize)]
pub struct FindMangaCoversByIdParams {
    #[serde(default = "<FindMangaCoversByIdParams as DefaultOffsetLimit>::default_offset")]
    pub offset: usize,
    #[serde(default = "<FindMangaCoversByIdParams as DefaultOffsetLimit>::default_limit")]
    pub limit: usize,
}

impl Default for FindMangaCoversByIdParams {
    fn default() -> Self {
        Self {
            offset: 0,
            limit: 10,
        }
    }
}

impl DefaultOffsetLimit<'_> for FindMangaCoversByIdParams {
    type OffsetOutput = usize;

    type LimitOutput = usize;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// find a downloaded covers manga
#[get("/manga/{id}/covers")]
pub async fn find_manga_covers_by_id(
    id: web::Path<String>,
    params: QsQuery<FindMangaCoversByIdParams>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let offset = params.offset;
    let limit = params.limit;
    let utils = app_state.manga_utils().with_id(format!("{}", id));
    let getted = utils
        .get_downloaded_cover_of_a_manga_collection(offset, limit, &mut app_state)
        .await?;
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : getted
        })
        .to_string(),
    ))
}

/// find a chapter (json data) by his id
#[get("/chapter/{id}/data")]
pub async fn find_chapters_data_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = &app_state.dir_options;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data", id).as_str());
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let filename_os = files?.file_name().clone();
            let filename = filename_os
                .to_str()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("can't reconize file").to_string(),
                ))?
                .to_string();
            if !filename.ends_with(".json") {
                vecs.push(filename);
            }
        }
        vecs = vecs.into_iter().unique().collect();
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : vecs
            })
            .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.to_string(),
            "message" : format!("Cannot find the chapter {} data in the api", id)
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}

/// find a chapters data-saver (json data) by his id
#[get("/chapter/{id}/data-saver")]
pub async fn find_chapters_data_saver_by_id(
    id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let file_dirs = &app_state.dir_options;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data-saver", id).as_str());
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let filename_os = files?.file_name().clone();
            let filename = filename_os
                .to_str()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    format!("can't reconize file"),
                ))?
                .to_string();
            if !filename.ends_with(".json") {
                vecs.push(filename);
            }
        }
        vecs = vecs.into_iter().unique().collect();
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                "result" : "ok",
                "type" : "collection",
                "data" : vecs
            })
            .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : format!("Cannot find the chapter {} data-saver in the api", id)
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
/// find a chapters data image by his id
#[get("/chapter/{id}/data/{filename}")]
pub async fn find_chapters_data_img_by_id(
    data: web::Path<(uuid::Uuid, String)>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let (id, filename) = data.into_inner();
    let file_dirs = &app_state.dir_options;
    let path = file_dirs.chapters_add(format!("{}/data/{}", id, filename).as_str());
    if Path::new(path.as_str()).exists() {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::jpeg())
            .body(std::fs::read(path)?))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.to_string(),
            "message" : format!("Cannot find the chapter {} data-saver in the api", id)
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}

/// find a chapters data-saver image by his id
#[get("/chapter/{id}/data-saver/{filename}")]
pub async fn find_chapters_data_saver_img_by_id(
    data: web::Path<(String, String)>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let (id, filename) = data.into_inner();
    let file_dirs = &app_state.dir_options;
    let path = file_dirs.chapters_add(format!("{}/data-saver/{}", id, filename).as_str());
    if Path::new(path.as_str()).exists() {
        Ok(HttpResponse::Ok()
            .content_type(ContentType::jpeg())
            .body(std::fs::read(path)?))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "type" : "manga",
            "id" : id.as_str(),
            "message" : format!("Cannot find the chapter {} data-saver in the api", id)
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}

/// find a chapter (json data) by his id
#[get("/chapter/{id}")]
pub async fn find_chapter_by_id(
    id: web::Path<String>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    //let path = format!("chapters/{}/data.json", id);
    let file_dirs = &app_state.dir_options;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data.json", id).as_str());
    if Path::new(path.as_str()).exists() {
        let jsons = std::fs::read_to_string(path.as_str())?;
        let mut history_ = app_state
            .get_history_w_file_by_rel_or_init(mangadex_api_types_rust::RelationshipType::Chapter)
            .await?;

        let uuid_str = format!("urn:uuid:{}", id);
        match uuid::Uuid::from_str(uuid_str.as_str()) {
            Ok(uuid_data) => {
                if history_.get_history().is_in(uuid_data) {
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

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GetChapterQuery {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub include_fails: Option<bool>,
    pub only_fails: Option<bool>,
}

impl Default for GetChapterQuery {
    fn default() -> Self {
        Self {
            offset: Some(0),
            limit: Some(10),
            include_fails: None,
            only_fails: None,
        }
    }
}

/// get all dowloaded chapter
#[get("/chapter")]
pub async fn find_all_downloaded_chapter(
    query: QsQuery<GetChapterQuery>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let mut app_state: AppState = From::from(app_state);
    let getted = app_state
        .chapter_utils()
        .get_all_downloaded_chapters(Some(query.into_inner()), &mut app_state)
        .await?;
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : getted
        })
        .to_string(),
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindAllDownloadedMangaParams {
    #[serde(default = "<FindAllDownloadedMangaParams as DefaultOffsetLimit>::default_offset")]
    pub offset: usize,
    #[serde(default = "<FindAllDownloadedMangaParams as DefaultOffsetLimit>::default_limit")]
    pub limit: usize,
    #[serde(default)]
    pub title: Option<String>,
}

impl DefaultOffsetLimit<'_> for FindAllDownloadedMangaParams {
    type OffsetOutput = usize;

    type LimitOutput = usize;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// find all downloaded manga
#[get("/manga")]
pub async fn find_all_downloaded_manga(
    params: QsQuery<FindAllDownloadedMangaParams>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let getted = app_state
        .manga_utils()
        .get_downloaded_manga(params.offset, params.limit, params.title.clone())
        .await?;

    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : getted
        })
        .to_string(),
    ))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FindMangaChaptersByIdParams {
    #[serde(default = "<FindMangaChaptersByIdParams as DefaultOffsetLimit>::default_offset")]
    pub offset: usize,
    #[serde(default = "<FindMangaChaptersByIdParams as DefaultOffsetLimit>::default_limit")]
    pub limit: usize,
}

impl DefaultOffsetLimit<'_> for FindMangaChaptersByIdParams {
    type OffsetOutput = usize;

    type LimitOutput = usize;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// find all downloaded chapter manga
#[get("/manga/{id}/chapters")]
pub async fn find_manga_chapters_by_id(
    id: web::Path<uuid::Uuid>,
    params: QsQuery<FindMangaChaptersByIdParams>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let to_use = app_state
        .manga_utils()
        .with_id(id.to_string())
        .get_downloaded_chapter(params.offset, params.limit)
        .await?;
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : to_use
        })
        .to_string(),
    ))
}

#[get("/manga/{id}/aggregate")]
pub async fn aggregate_manga(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let aggregate = app_state
        .manga_utils()
        .with_id(id.to_string())
        .aggregate_manga_chapters()
        .await?;
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "volumes" : aggregate.volumes
        })
        .to_string(),
    ))
}
