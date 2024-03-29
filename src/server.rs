mod app_state;
pub mod traits;
#[cfg(feature = "actix_web")]
use crate::methods::delete::{delete_chapter_by_id, delete_manga_chapters_by_id};
#[cfg(feature = "actix_web")]
use crate::methods::get::{
    aggregate_manga, find_all_downloaded_chapter, find_all_downloaded_manga, find_chapter_by_id,
    find_chapters_data_by_id, find_chapters_data_img_by_id, find_chapters_data_saver_by_id,
    find_chapters_data_saver_img_by_id, find_cover_by_id, find_cover_image_by_id, find_manga_by_id,
    find_manga_chapters_by_id, find_manga_cover_by_id, find_manga_covers_by_id, hello,
};
#[cfg(feature = "actix_web")]
use crate::methods::patch::{
    patch_all_chapter, patch_all_chapter_manga, patch_all_manga_cover, update_chapter_by_id,
    update_chapter_manga_by_id, update_cover_by_id,
};
#[cfg(feature = "actix_web")]
use crate::methods::put::{
    download_chapter_byid, download_chapter_data_byid, download_chapter_data_saver_byid,
    download_cover, download_cover_quality, download_manga_by_id, download_manga_cover,
    download_manga_cover_quality, download_manga_covers,
};
#[cfg(feature = "actix_web")]
use actix_cors::Cors;
#[cfg(feature = "actix_web")]
use actix_web::body::MessageBody;
#[cfg(feature = "actix_web")]
use actix_web::dev::{self, Server, ServiceFactory, ServiceRequest, ServiceResponse};
#[cfg(feature = "actix_web")]
use actix_web::http::header::{self};
#[cfg(feature = "actix_web")]
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
#[cfg(feature = "actix_web")]
use actix_web::{
    http::StatusCode,
    App,
    HttpServer,
    //web
};
#[cfg(feature = "actix_web")]
use actix_web::{web, Error};
pub use app_state::AppState;
#[cfg(feature = "unix-socket-support")]
mod unix;
#[cfg(feature = "unix-socket-support")]
pub use unix::launch_async_server_with_unix_socket;
/*use self::state::AppState;
use std::collections::HashMap;
use std::sync::Arc;
use crate::settings::files_dirs::DirsOptions;
pub mod state;
*/
/// url not found handler
///
///
///
#[cfg(feature = "actix_web")]
fn not_found_message<B>(
    mut res: dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    let (req, res) = res.into_parts();
    let json = serde_json::json!({
        "result" : "error",
        "message" : format!("Ressource {} {} not found", req.method(), req.path())
    });
    let res = res.set_body(json.to_string());
    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();
    Ok(ErrorHandlerResponse::Response(res))
}
#[cfg(feature = "actix_web")]
fn not_allowed_message<B>(
    mut res: dev::ServiceResponse<B>,
) -> Result<ErrorHandlerResponse<B>, actix_web::Error> {
    res.response_mut().headers_mut().insert(
        header::CONTENT_TYPE,
        header::HeaderValue::from_static("application/json"),
    );
    let (req, res) = res.into_parts();
    let json = serde_json::json!({
        "result" : "error",
        "type" : "Not Allowed",
        "message" : format!("Ressource {} {} is still busy! Please call it later", req.method(), req.path())
    });
    let res = res.set_body(json.to_string());
    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();
    Ok(ErrorHandlerResponse::Response(res))
}
#[cfg(feature = "actix_web")]
pub fn get_actix_app(
    app_state: web::Data<AppState>,
) -> App<
    impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<impl MessageBody>,
            Error = Error,
            InitError = (),
        > + 'static,
> {
    let cors = Cors::default().allow_any_origin().send_wildcard();
    App::new()
        .app_data(app_state)
        .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found_message))
        .wrap(ErrorHandlers::new().handler(StatusCode::METHOD_NOT_ALLOWED, not_allowed_message))
        .wrap(cors)
        /*
            get Methods
        */
        .service(find_manga_by_id)
        .service(find_cover_by_id)
        .service(find_cover_image_by_id)
        .service(find_manga_cover_by_id)
        .service(find_manga_covers_by_id)
        .service(find_chapters_data_by_id)
        .service(find_chapters_data_saver_by_id)
        .service(find_chapters_data_img_by_id)
        .service(find_chapters_data_saver_img_by_id)
        .service(find_chapter_by_id)
        .service(find_all_downloaded_chapter)
        .service(find_all_downloaded_manga)
        .service(find_manga_chapters_by_id)
        .service(aggregate_manga)
        .service(hello)
        /*
            patch methods
        */
        .service(update_cover_by_id)
        .service(update_chapter_by_id)
        .service(patch_all_chapter)
        .service(patch_all_chapter_manga)
        .service(update_chapter_manga_by_id)
        .service(patch_all_manga_cover)
        /*
            delete methods
        */
        .service(delete_chapter_by_id)
        .service(delete_manga_chapters_by_id)
        /*
            put methods
        */
        .service(download_manga_by_id)
        .service(download_manga_covers)
        .service(download_manga_cover)
        .service(download_manga_cover_quality)
        .service(download_cover)
        .service(download_cover_quality)
        .service(download_chapter_byid)
        .service(download_chapter_data_byid)
        .service(download_chapter_data_saver_byid)
}

#[cfg(feature = "actix_web")]
/// Get the server handle
pub fn launch_async_server(
    app_state: AppState,
    (address, port): (String, u16),
) -> std::io::Result<Server> {
    let app_state_ref = web::Data::new(app_state);
    Ok(
        HttpServer::new(move || get_actix_app(app_state_ref.clone()))
            .bind((address, port))?
            .run(),
    )
}
