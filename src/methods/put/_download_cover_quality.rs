use crate::core::ManagerCoreResult;
use crate::download::cover::AccessCoverDownload;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{put, web, HttpResponse, Responder};
use mangadex_api::utils::download::cover::CoverQuality;

/// download cover by id with defined quality
#[put("/cover/{id}/{quality}")]
pub async fn download_cover_quality(
    path: web::Path<(uuid::Uuid, u32)>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let (id, quality) = path.into_inner();
    let cover_download = app_state.cover_download(id);
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
