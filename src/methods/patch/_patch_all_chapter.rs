use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::chapter::AccessChapterUtisWithID;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use log::info;
use tokio_stream::StreamExt;

/// update all chapters data
#[patch("/chapter/all")]
pub async fn patch_all_chapter(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state.clone());
    let chapter_utils = app_state.chapter_utils();
    let mut stream = Box::pin(chapter_utils.get_all_chapter_without_history()?);
    let mut vecs: Vec<uuid::Uuid> = Vec::new();
    while let Some(id) = stream.next().await {
        let util = app_state.chapter_utils().with_id(id);
        vecs.push(
            <AppState as AccessChapterUtisWithID>::update(&mut app_state, &util)
                .await?
                .data
                .id,
        );
        info!("downloaded chapter data {}", id);
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
