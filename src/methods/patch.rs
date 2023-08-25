use crate::core::ManagerCoreResult;
use crate::download::cover::{AccessCoverDownloadWithManga, CoverDownloadWithManga};
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use log::info;
use std::path::Path;

// NOTE update api data

/// update a cover json data by his id
#[patch("/cover/{id}")]
pub async fn update_cover_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    app_state
        .cover_download(*id)
        .download(&mut app_state)
        .await?;

    let returns = app_state.cover_utils().with_id(id.to_string()).get_data()?;

    Ok(HttpResponse::Ok().json(returns))
}

/// update a chapter by his id
#[patch("/chapter/{id}")]
pub async fn update_chapter_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state.clone());
    let utils = app_state.chapter_utils().with_id(id.to_string());
    let data = <AppState as AccessChapterUtisWithID>::update(&mut app_state, &utils).await?;
    Ok(HttpResponse::Ok().json(data))
}

/// update all chapters data
#[patch("/chapter/all")]
pub async fn patch_all_chapter(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state.clone());
    let path = app_state.dir_options.chapters_add("");
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<uuid::Uuid> = Vec::new();
        for files in list_dir.flatten() {
            if let Ok(file_type) = files.file_type() {
                if file_type.is_dir() {
                    let id = files
                        .file_name()
                        .to_str()
                        .ok_or(std::io::Error::new(
                            std::io::ErrorKind::InvalidData,
                            "can't reconize file".to_string(),
                        ))?
                        .to_string();
                    let util = app_state.chapter_utils().with_id(id.clone());
                    vecs.push(
                        <AppState as AccessChapterUtisWithID>::update(&mut app_state, &util)
                            .await?
                            .data
                            .id,
                    );
                    info!("downloaded chapter data {}", id);
                }
            }
        }
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                    "result" : "ok",
                    "tasks" : "patched",
                    "type" : "collection",
                    "data" : vecs
            })
            .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters directory"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}

/// patch all chapters manga data
#[patch("/chapter/all/patch-manga")]
pub async fn patch_all_chapter_manga(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let path = app_state.dir_options.chapters_add("");
    //info!("{}", path);
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir.flatten() {
            if let Some(id) = files.file_name().to_str() {
                let utils = app_state.chapter_utils().with_id(id.to_string());
                if let Ok(is_there) = utils.is_manga_there() {
                    if !is_there {
                        vecs.push(app_state.patch_manga(&utils).await?);
                        info!("downloaded manga data {}", id.to_string());
                    }
                }
            }
        }
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            serde_json::json!({
                "result" : "ok",
                "tasks" : "patched",
                "type" : "collection",
                "data" : vecs
            })
            .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters directory"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}

/// patch a chapter manga data
#[patch("/chapter/{id}/patch-manga")]
pub async fn update_chapter_manga_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);

    let path = app_state
        .dir_options
        .chapters_add(format!("chapters/{}/data.json", id).as_str());
    let utils = app_state.chapter_utils().with_id(id.to_string());
    if Path::new(path.as_str()).exists() {
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            AccessChapterUtisWithID::patch_manga(&mut app_state, &utils)
                .await?
                .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters data"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}

/// patch all manga cover
#[patch("/manga/all/cover")]
pub async fn patch_all_manga_cover(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);

    let path = app_state.dir_options.mangas_add("");
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir.flatten() {
            let manga_id = files.file_name().to_str().ok_or(std::io::Error::new(std::io::ErrorKind::InvalidData, "can't reconize file".to_string()))?.to_string().replace(".json", "");
            let manga_cover_download : CoverDownloadWithManga = TryFrom::try_from(app_state.manga_utils().with_id(manga_id))?;
            if let Ok(result) = <AppState as AccessCoverDownloadWithManga>::download(&mut app_state, &manga_cover_download).await {
                vecs.push(result);
            }
        }
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
        "message" : "can't find the chapters directory"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
