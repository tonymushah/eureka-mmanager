use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::ExtractData;
use actix_web::{get, web, HttpResponse, Responder};
use mangadex_api_schema_rust::ApiData;
use mangadex_api_types_rust::{ResponseType, ResultType};

/// find a cover by his id
/// # How to use
/// {app deployed url}/cover/{cover_id}
#[get("/cover/{id}")]
pub async fn find_cover_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let data = app_state.cover_utils().with_id(*id).get_data()?;
    let value = ApiData {
        data,
        result: ResultType::Ok,
        response: ResponseType::Entity,
    };
    Ok(HttpResponse::Ok().json(value))
}
