use crate::core::ManagerCoreResult;
use crate::download::chapter::AccessChapterDownload;
use crate::server::AppState;
use actix_web::{put, web, HttpResponse, Responder};

/// download chapter data-saver by id
#[put("/chapter/{id}/data-saver")]
pub async fn download_chapter_data_saver_byid(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let chapter_download = app_state.chapter_download(*id);
    let response =
        <AppState as AccessChapterDownload>::download_data_saver(&mut app_state, &chapter_download)
            .await?;
    Ok(HttpResponse::Ok().json(response))
}
