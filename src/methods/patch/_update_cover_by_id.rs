use crate::server::AppState;
use crate::{core::ManagerCoreResult, utils::ExtractData};
use actix_web::{patch, web, HttpResponse, Responder};

// NOTE update api data

/// update a cover json data by his id
#[patch("/cover/{id}")]
pub async fn update_cover_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state);
    app_state
        .cover_download(*id)
        .download(&mut app_state)
        .await?;

    let returns = app_state.cover_utils().with_id(*id).get_data()?;

    Ok(HttpResponse::Ok().json(returns))
}
