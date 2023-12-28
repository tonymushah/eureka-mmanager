use crate::server::AppState;
use crate::utils::chapter::{AccessChapterUtisWithID, ChapterUtilsWithID};
use crate::{core::ManagerCoreResult, utils::ExtractData};
use actix_web::http::header::{ContentType, CONTENT_TYPE};
use actix_web::web::Payload;
use actix_web::{patch, web, HttpRequest, HttpResponse, Responder};
use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_schema_rust::ApiData;
use mangadex_api_types_rust::{ResponseType, ResultType};

/// update a chapter by his id
#[patch("/chapter/{id}")]
pub async fn update_chapter_by_id(
    id: web::Path<uuid::Uuid>,
    app_state: web::Data<AppState>,
    request: HttpRequest,
    payload: Payload,
) -> ManagerCoreResult<impl Responder> {
    let mut app_state: AppState = From::from(app_state.clone());
    let utils = app_state.chapter_utils().with_id(*id);
    if request
        .headers()
        .iter()
        .any(|(k, v)| *k == CONTENT_TYPE && *v == ContentType::json().0.as_ref())
    {
        let input: ChapterObject = serde_json::from_slice(
            &payload
                .to_bytes()
                .await
                .map_err(|e| anyhow::Error::msg(e.to_string()))?,
        )?;
        <ChapterUtilsWithID as ExtractData>::update(&utils, input)?;
        let data = ApiData {
            response: ResponseType::Entity,
            result: ResultType::Ok,
            data: utils.get_data()?,
        };
        Ok(HttpResponse::Ok().json(data))
    } else {
        let data = <AppState as AccessChapterUtisWithID>::update(&mut app_state, &utils).await?;
        Ok(HttpResponse::Ok().json(data))
    }
}
