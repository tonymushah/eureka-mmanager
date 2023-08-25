use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::{patch, web, HttpResponse, Responder};

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
