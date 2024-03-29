#[cfg(feature = "actix_web")]
use crate::core::Error;
#[cfg(feature = "actix_web")]
use crate::server::AppState;
#[cfg(feature = "actix_web")]
use actix_web::{get, web, HttpResponse, Responder};
use mangadex_api_input_types::chapter::list::ChapterListParams;
#[cfg(feature = "actix_web")]
use serde_qs::actix::QsQuery;

#[derive(serde::Deserialize, Clone, Default)]
pub struct GetChapterQuery {
    #[serde(flatten)]
    pub params: ChapterListParams,
    pub include_fails: Option<bool>,
    pub only_fails: Option<bool>,
}

/// get all dowloaded chapter
#[cfg(feature = "actix_web")]
#[get("/chapter")]
pub async fn find_all_downloaded_chapter(
    query: QsQuery<GetChapterQuery>,
    app_state: web::Data<AppState>,
) -> Result<impl Responder, Error> {
    let mut app_state: AppState = From::from(app_state);
    //log::info!("getted reqwest");
    let getted = app_state
        .chapter_utils()
        .get_all_downloaded_chapters(Some(query.into_inner()), &mut app_state)
        .await?;
    Ok(HttpResponse::Ok().json(getted))
}
