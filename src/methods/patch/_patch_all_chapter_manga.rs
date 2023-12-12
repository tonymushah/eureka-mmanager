use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use log::info;
use tokio_stream::StreamExt;
use uuid::Uuid;

/// patch all chapters manga data
#[patch("/chapter/all/patch-manga")]
pub async fn patch_all_chapter_manga(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let ch_u = app_state.chapter_utils();
    let mut stream = Box::pin(ch_u.get_all_chapter_without_history()?);

    let mut vecs: Vec<Uuid> = Vec::new();
    while let Some(id) = stream.next().await {
        let utils = app_state.chapter_utils().with_id(id);
        if let Ok(is_there) = utils.is_manga_there() {
            if !is_there && (app_state.patch_manga(&utils).await).is_ok() {
                vecs.push(id);
                info!("downloaded manga data {}", id);
            }
        }
    }
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "tasks" : "patched",
            "type" : "collection",
            "data" : vecs
        })
        .to_string(),
    ))
}
