use crate::core::ManagerCoreResult;
use crate::download::cover::{AccessCoverDownloadWithManga, CoverDownloadWithManga};
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{patch, web, HttpResponse, Responder};
use std::path::Path;

/// patch all manga cover
#[patch("/manga/all/cover")]
pub async fn patch_all_manga_cover(
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);

    let path = app_state.dir_options.mangas_add("");
    if Path::new(path.as_str()).exists() {
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<serde_json::Value> = Vec::new();
        for files in list_dir.flatten() {
            let manga_id = files
                .file_name()
                .to_str()
                .ok_or(std::io::Error::new(
                    std::io::ErrorKind::InvalidData,
                    "can't reconize file".to_string(),
                ))?
                .to_string()
                .replace(".json", "");
            let manga_cover_download: CoverDownloadWithManga =
                TryFrom::try_from(app_state.manga_utils().with_id(manga_id))?;
            if let Ok(result) = <AppState as AccessCoverDownloadWithManga>::download(
                &mut app_state,
                &manga_cover_download,
            )
            .await
            {
                vecs.push(result);
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
    } else {
        let jsons = serde_json::json!({
        "result" : "error",
        "message" : "can't find the chapters directory"
        });
        Ok(HttpResponse::NotFound()
            .content_type(ContentType::json())
            .body(jsons.to_string()))
    }
}