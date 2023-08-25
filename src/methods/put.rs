use crate::core::ManagerCoreResult;
use crate::download::chapter::AccessChapterDownload;
use crate::download::cover::{
    AccessCoverDownload, AccessCoverDownloadWithManga, CoverDownloadWithManga,
};
use crate::download::manga::AccessMangaDownload;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{put, web, HttpResponse, Responder};
use mangadex_api::utils::download::cover::CoverQuality;
use serde::{Deserialize, Serialize};
use serde_qs::actix::QsQuery;

use super::DefaultOffsetLimit;

// NOTE All download methods

/// download a manga (req only)
#[put("/manga/{id}")]
pub async fn download_manga_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let manga_download = app_state.manga_download(id.clone());
    <AppState as AccessMangaDownload>::download(&mut app_state, &manga_download).await?;
    let jsons = serde_json::json!({
        "result" : "ok",
        "type" : "manga",
        "id" : id.to_string()
    });
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(jsons.to_string()))
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadMangaCoversParams {
    #[serde(default = "<DownloadMangaCoversParams as DefaultOffsetLimit>::default_limit")]
    pub limit: u32,
}

impl DefaultOffsetLimit<'_> for DownloadMangaCoversParams {
    type OffsetOutput = u32;

    type LimitOutput = u32;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// download all manga covers
#[put("/manga/{id}/covers")]
pub async fn download_manga_covers(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
    params: QsQuery<DownloadMangaCoversParams>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let manga_cover_download: CoverDownloadWithManga =
        From::from(app_state.manga_download(id.clone()));
    let response = <AppState as AccessCoverDownloadWithManga>::all_cover_download(
        &mut app_state,
        &manga_cover_download,
        params.limit,
    )
    .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}

/// download the top manga cover
#[put("/manga/{id}/cover")]
pub async fn download_manga_cover(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let cover_download: CoverDownloadWithManga = From::from(app_state.manga_download(id.clone()));
    let response =
        <AppState as AccessCoverDownloadWithManga>::download(&mut app_state, &cover_download)
            .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}

/// download the top manga cover with defined quality
#[put("/manga/{id}/cover/{quality}")]
pub async fn download_manga_cover_quality(
    path: web::Path<(uuid::Uuid, u32)>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let (id, quality) = path.into_inner();
    let mut app_state: AppState = From::from(app_state);
    let manga_cover_download: CoverDownloadWithManga = From::from(app_state.manga_download(id));
    let response = <AppState as AccessCoverDownloadWithManga>::download_with_quality(
        &mut app_state,
        &manga_cover_download,
        match quality {
            256 => CoverQuality::Size256,
            512 => CoverQuality::Size512,
            _ => CoverQuality::Default,
        },
    )
    .await?;

    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}

/// download cover by id
#[put("/cover/{id}")]
pub async fn download_cover(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let cover_download = app_state.cover_download(id.clone());
    let response =
        <AppState as AccessCoverDownload>::download(&mut app_state, &cover_download).await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}

/// download cover by id with defined quality
#[put("/cover/{id}/{quality}")]
pub async fn download_cover_quality(
    path: web::Path<(uuid::Uuid, u32)>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let (id, quality) = path.into_inner();
    let cover_download = app_state.cover_download(id.clone());
    let response = <AppState as AccessCoverDownload>::download_with_quality(
        &mut app_state,
        &cover_download,
        match quality {
            256 => CoverQuality::Size256,
            512 => CoverQuality::Size512,
            _ => CoverQuality::Default,
        },
    )
    .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}

/// download chapter by id
#[put("/chapter/{id}")]
pub async fn download_chapter_byid(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let chapter_download = app_state.chapter_download(id.clone());
    let response =
        <AppState as AccessChapterDownload>::download(&mut app_state, &chapter_download).await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}

/// download chapter data by id
#[put("/chapter/{id}/data")]
pub async fn download_chapter_data_byid(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let chapter_download = app_state.chapter_download(id.clone());
    let response =
        <AppState as AccessChapterDownload>::download(&mut app_state, &chapter_download).await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}

/// download chapter data-saver by id
#[put("/chapter/{id}/data-saver")]
pub async fn download_chapter_data_saver_byid(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let chapter_download = app_state.chapter_download(id.clone());
    let response =
        <AppState as AccessChapterDownload>::download_data_saver(&mut app_state, &chapter_download)
            .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}
