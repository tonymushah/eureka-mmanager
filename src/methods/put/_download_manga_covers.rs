use crate::core::ManagerCoreResult;
use crate::download::cover::{AccessCoverDownloadWithManga, CoverDownloadWithManga};
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{put, web, HttpResponse, Responder};
use serde::{Deserialize, Serialize};
use serde_qs::actix::QsQuery;

use super::DefaultOffsetLimit;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DownloadMangaCoversParams {
    #[serde(default = "<DownloadMangaCoversParams as DefaultOffsetLimit>::default_limit")]
    pub limit: u32,
}

impl DefaultOffsetLimit<'_> for DownloadMangaCoversParams {
    type OffsetOutput = u32;

    type LimitOutput = u32;

    fn default_offset() -> Self::OffsetOutput {
        0
    }

    fn default_limit() -> Self::LimitOutput {
        10
    }
}

/// download all manga covers
#[put("/manga/{id}/covers")]
pub async fn download_manga_covers(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
    params: QsQuery<DownloadMangaCoversParams>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    let manga_cover_download: CoverDownloadWithManga = From::from(app_state.manga_download(*id));
    let response = <AppState as AccessCoverDownloadWithManga>::all_cover_download(
        &mut app_state,
        &manga_cover_download,
        params.limit,
    )
    .await?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::json())
        .body(response.to_string()))
}
