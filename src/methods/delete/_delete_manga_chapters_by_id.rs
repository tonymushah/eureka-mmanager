use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{delete, web, HttpResponse, Responder};

/// delete a  manga
/// TODO Refactor this function to utils
#[delete("/manga/{id}")]
pub async fn delete_manga_chapters_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    app_state.manga_utils().with_id(*id).delete().await?;

    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            //"data" : jsons,
            "message" : "deleted "
        })
        .to_string(),
    ))
}