use crate::core::ManagerCoreResult;
use crate::server::AppState;
use actix_web::http::header::ContentType;
use actix_web::{get, web, HttpResponse, Responder};
use std::io::Read;

/// find a cover by his id
#[get("/cover/{id}/image")]
pub async fn find_cover_image_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let utils = app_state.cover_utils().with_id(*id);
    let image_path = utils.get_image_path()?;
    let mut buf_read = utils.get_image_buf_reader()?;
    let mut response = HttpResponse::Ok();
    if let Some(image_ext) = image_path.extension() {
        if image_ext == "png" {
            response.content_type(ContentType::png());
        } else {
            response.content_type(ContentType::jpeg());
        }
    } else {
        response.content_type(ContentType::jpeg());
    }
    let mut bytes = Vec::<u8>::new();
    buf_read.read_to_end(&mut bytes)?;
    Ok(response.body(bytes))
}
