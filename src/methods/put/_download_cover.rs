use crate::core::ManagerCoreResult;
use crate::download::cover::AccessCoverDownload;
use crate::server::AppState;
use actix_web::{put, web, HttpResponse, Responder};

/// download cover by id
#[put("/cover/{id}")]
pub async fn download_cover(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let cover_download = app_state.cover_download(*id);
    let response =
        <AppState as AccessCoverDownload>::download(&mut app_state, &cover_download).await?;
    Ok(HttpResponse::Ok().json(response))
}
