use crate::core::ManagerCoreResult;
use crate::server::AppState;
use crate::settings::file_history::{HistoryEntry, NoLFAsyncIsIn};
use crate::utils::ExtractData;
use actix_web::{get, web, HttpResponse, Responder};
use mangadex_api_schema_rust::ApiData;
use mangadex_api_types_rust::{ResponseType, ResultType};
use uuid::Uuid;

/// find a chapter (json data) by his id
#[get("/chapter/{id}")]
pub async fn find_chapter_by_id(
    id: web::Path<Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let data = ApiData {
        data: app_state.chapter_utils().with_id(*id).get_data()?,
        response: ResponseType::Entity,
        result: ResultType::Ok,
    };
    if <AppState as NoLFAsyncIsIn<HistoryEntry>>::is_in(
        &app_state,
        HistoryEntry::new(*id, mangadex_api_types_rust::RelationshipType::Chapter),
    )
    .await?
    {
        Ok(HttpResponse::Ok()
            .insert_header(("X-DOWNLOAD-FAILED", "true"))
            .json(data))
    } else {
        Ok(HttpResponse::Ok()
            .insert_header(("X-DOWNLOAD-FAILED", "false"))
            .json(data))
    }
}
