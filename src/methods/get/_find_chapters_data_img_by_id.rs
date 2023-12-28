use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use bytes::BytesMut;
use std::io::Read;
use uuid::Uuid;

use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ThisPath {
    id: Uuid,
    filename: String,
}

/// find a chapters data image by his id
#[get("/chapter/{id}/data/{filename}")]
pub async fn find_chapters_data_img_by_id(
    data: web::Path<ThisPath>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut buf_reader = app_state
        .chapter_utils()
        .with_id(data.id)
        .get_data_image(&data.filename)?;
    let mut buf: BytesMut = BytesMut::new();
    buf_reader.read_exact(&mut buf)?;
    Ok(HttpResponse::Ok()
        .content_type(ContentType::jpeg())
        .body(buf))
}
