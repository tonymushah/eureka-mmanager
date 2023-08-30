use crate::core::Error;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use serde_qs::actix::QsQuery;

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct GetChapterQuery {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
    pub include_fails: Option<bool>,
    pub only_fails: Option<bool>,
}

impl Default for GetChapterQuery {
    fn default() -> Self {
        Self {
            offset: Some(0),
            limit: Some(10),
            include_fails: None,
            only_fails: None,
        }
    }
}

/// get all dowloaded chapter
#[get("/chapter")]
pub async fn find_all_downloaded_chapter(
    query: QsQuery<GetChapterQuery>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let mut app_state: AppState = From::from(app_state);
    let getted = app_state
        .chapter_utils()
        .get_all_downloaded_chapters(Some(query.into_inner()), &mut app_state)
        .await?;
    Ok(HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "type" : "collection",
            "data" : getted
        })
        .to_string(),
    ))
}