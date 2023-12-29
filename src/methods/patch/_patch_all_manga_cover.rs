use crate::core::ManagerCoreResult;
use crate::download::cover::{AccessCoverDownloadWithManga, CoverDownloadWithManga};
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use tokio_stream::StreamExt;
use uuid::Uuid;

/// patch all manga cover
#[patch("/manga/all/cover")]
pub async fn patch_all_manga_cover(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let manga_utils = app_state.manga_utils();
    let mut stream = Box::pin(manga_utils.get_all_downloaded_manga()?);
    let mut vecs: Vec<Uuid> = Vec::new();
    while let Some(id) = stream.next().await {
        let manga_cover_download: CoverDownloadWithManga =
            From::from(app_state.manga_utils().with_id(id));
        if let Ok(result) = <AppState as AccessCoverDownloadWithManga>::download(
            &mut app_state,
            &manga_cover_download,
        )
        .await
        {
            vecs.push(result.data.id);
        }
    }
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : vecs
        })
        .to_string(),
    ))
}
