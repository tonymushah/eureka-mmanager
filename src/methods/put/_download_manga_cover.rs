use crate::core::ManagerCoreResult;
use crate::download::cover::{AccessCoverDownloadWithManga, CoverDownloadWithManga};
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{put, web, HttpResponse, Responder};

/// download the top manga cover
#[put("/manga/{id}/cover")]
pub async fn download_manga_cover(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let cover_download: CoverDownloadWithManga = From::from(app_state.manga_download(*id));
    let response =
        <AppState as AccessCoverDownloadWithManga>::download(&mut app_state, &cover_download)
            .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}
