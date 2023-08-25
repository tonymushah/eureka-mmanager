use crate::core::ManagerCoreResult;
use crate::download::cover::{AccessCoverDownloadWithManga, CoverDownloadWithManga};
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{put, web, HttpResponse, Responder};
use mangadex_api::utils::download::cover::CoverQuality;

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
