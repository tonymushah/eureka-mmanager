use crate::core::ManagerCoreResult;
use crate::download::chapter::AccessChapterDownload;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{put, web, HttpResponse, Responder};

/// download chapter by id
#[put("/chapter/{id}")]
pub async fn download_chapter_byid(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let chapter_download = app_state.chapter_download(*id);
    let response = <AppState as AccessChapterDownload>::download(&mut app_state, &chapter_download).await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}
