use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::ExtractData;
use actix_web::{get, web, HttpResponse, Responder};
use mangadex_api_schema_rust::ApiData;
use mangadex_api_types_rust::{ResponseType, ResultType};

/// Find a downloaded manga
/// # How to use
/// {app deployed url}/manga/{manga_id}
#[get("/manga/{id}")]
pub async fn find_manga_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    Ok(HttpResponse::Ok().json(ApiData {
        response: ResponseType::Entity,
        result: ResultType::Ok,
        data: app_state.manga_utils().with_id(*id).get_data()?,
    }))
}
