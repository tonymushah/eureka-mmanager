use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use std::path::Path;

/// patch a chapter manga data
#[patch("/chapter/{id}/patch-manga")]
pub async fn update_chapter_manga_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);

    let path = app_state
        .dir_options
        .chapters_add(format!("chapters/{}/data.json", id).as_str());
    let utils = app_state.chapter_utils().with_id(id.to_string());
    if Path::new(path.as_str()).exists() {
        Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
            AccessChapterUtisWithID::patch_manga(&mut app_state, &utils)
                .await?
                .to_string(),
        ))
    } else {
        let jsons = serde_json::json!({
            "result" : "error",
            "message" : "can't find the chapters data"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}
