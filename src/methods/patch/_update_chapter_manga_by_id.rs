use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::{patch, web, HttpResponse, Responder};

/// patch a chapter manga data
#[patch("/chapter/{id}/patch-manga")]
pub async fn update_chapter_manga_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);

    let utils = app_state.chapter_utils().with_id(*id);
    AccessChapterUtisWithID::patch_manga(&mut app_state, &utils)
            .await?;
    Ok(HttpResponse::Ok().json(
        serde_json::json!({
            "result" : "ok",
            "type" : "manga",
            "id" : *id
        })
    ))
}
