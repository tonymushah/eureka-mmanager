use crate::core::ManagerCoreResult;
use crate::download::manga::AccessMangaDownload;
use crate::server::AppState;
use actix_web::{put, web, HttpResponse, Responder};

/// download a manga (req only)
#[put("/manga/{id}")]
pub async fn download_manga_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let manga_download = app_state.manga_download(*id);
    let value =
        <AppState as AccessMangaDownload>::download(&mut app_state, &manga_download).await?;

    Ok(HttpResponse::Ok().json(value))
}
