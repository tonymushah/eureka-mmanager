use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use std::io::Read;
use uuid::Uuid;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ThisPath {
    id: Uuid,
    filename: String,
}

/// find a chapters data-saver image by his id
#[get("/chapter/{id}/data-saver/{filename}")]
pub async fn find_chapters_data_saver_img_by_id(
    data: web::Path<ThisPath>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut buf_reader = app_state
        .chapter_utils()
        .with_id(data.id)
        .get_data_saver_image(&data.filename)?;
    let mut buf = Vec::<u8>::new();
    buf_reader.read_to_end(&mut buf)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(buf))
}
