use std::sync::Arc;

use crate::methods::delete::{delete_chapter_by_id, delete_manga_chapters_by_id};
use crate::methods::get::{
    aggregate_manga, find_all_downloaded_chapter, find_all_downloaded_manga, find_chapter_by_id,
    find_chapters_data_by_id, find_chapters_data_img_by_id, find_chapters_data_saver_by_id,
    find_chapters_data_saver_img_by_id, find_cover_by_id, find_cover_image_by_id, find_manga_by_id,
    find_manga_chapters_by_id, find_manga_cover_by_id, find_manga_covers_by_id, hello,
};
use crate::methods::patch::{
    patch_all_chapter, patch_all_chapter_manga, patch_all_manga_cover, update_chapter_by_id,
    update_chapter_manga_by_id, update_cover_by_id,
};
use crate::methods::put::{
    download_chapter_byid, download_chapter_data_byid, download_chapter_data_saver_byid,
    download_cover, download_cover_quality, download_manga_by_id, download_manga_cover,
    download_manga_cover_quality, download_manga_covers,
};
use actix_web::body::MessageBody;
use actix_web::dev::{self, Server, ServiceFactory, ServiceRequest, ServiceResponse};
use actix_web::http::header::{self};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{
    http::StatusCode,
    App,
    HttpServer,
    //web
};
use actix_web::{web, Error};
use futures::lock::Mutex;
use mangadex_api::{HttpClient, HttpClientRef};
#[cfg(feature = "unix-socket-support")]
mod unix;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
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

#[derive(Clone)]
pub struct AppState {
    pub http_client: HttpClientRef,
}

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
        "message" : format!("Ressource {} not found", req.path())
    });
    let res = res.set_body(json.to_string());
    let res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();
    Ok(ErrorHandlerResponse::Response(res))
}

pub fn get_actix_app(
    client: Client,
) -> App<
    impl ServiceFactory<
            ServiceRequest,
            Config = (),
            Response = ServiceResponse<impl MessageBody>,
            Error = Error,
            InitError = (),
        > + 'static,
> {
    let state = AppState {
        http_client: Arc::new(Mutex::new(HttpClient::new(client))),
    };
    App::new()
        .app_data(web::Data::new(state))
        .wrap(ErrorHandlers::new().handler(StatusCode::NOT_FOUND, not_found_message))
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

/// Get the server handle
pub fn launch_async_server(address: &str, port: u16) -> std::io::Result<Server> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "User-Agent",
        HeaderValue::from_static("special-eureka-manager/0.4.0"),
    );
    let client = match Client::builder().default_headers(headers).build() {
        Ok(c) => c,
        Err(e) => {
            return std::io::Result::Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        }
    };
    Ok(HttpServer::new(move|| get_actix_app(client.clone()))
        .bind((address, port))?
        .run())
}
