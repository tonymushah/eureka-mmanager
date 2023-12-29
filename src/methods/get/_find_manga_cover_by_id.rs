use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::utils::cover::CoverUtilsWithId;
use crate::utils::ExtractData;
use actix_web::{get, web, HttpResponse, Responder};
use mangadex_api_schema_rust::ApiData;
use mangadex_api_types_rust::ResponseType;
use uuid::Uuid;

/// find a downloaded manga cover
#[get("/manga/{id}/cover")]
pub async fn find_manga_cover_by_id(
    id: web::Path<Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let cover_utils: CoverUtilsWithId = app_state.manga_utils().with_id(*id).try_into()?;
    let data = cover_utils.get_data()?;
    let value = ApiData {
        response: ResponseType::Entity,
        result: Default::default(),
        data,
    };
    Ok(HttpResponse::Ok().json(value))
}
