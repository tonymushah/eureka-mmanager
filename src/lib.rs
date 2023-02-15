use crate::settings::file_history::{load_history};
use crate::settings::{
    initialise_data_dir, initialise_settings_dir, verify_data_dir, verify_settings_dir,
};
use actix_web::dev::{self, Server, ServiceResponse};
use actix_web::http::header::{self};
use actix_web::middleware::{ErrorHandlerResponse, ErrorHandlers};
use actix_web::{
    http::StatusCode, App, HttpServer,
};
use log::{info, warn};
use settings::server_options;
use methods::get::{find_all_downloaded_manga, find_cover_by_id, find_chapters_data_saver_img_by_id, find_manga_by_id, find_cover_image_by_id, find_manga_cover_by_id, find_manga_covers_by_id, find_chapters_data_by_id, find_chapters_data_saver_by_id, find_chapters_data_img_by_id, find_chapter_by_id, find_all_downloaded_chapter, find_manga_chapters_by_id, hello};
use methods::patch::{update_cover_by_id, update_chapter_by_id, patch_all_chapter, patch_all_chapter_manga, update_chapter_manga_by_id, patch_all_manga_cover};
use methods::delete::{delete_chapter_by_id, delete_manga_chapters_by_id};
use methods::put::{download_chapter_data_saver_byid, download_chapter_data_byid, download_chapter_byid, download_cover_quality, download_cover, download_manga_cover_quality, download_manga_cover, download_manga_covers, download_manga_by_id};

mod methods;
pub mod chapter_download;
pub mod cover_download;
pub mod manga_download;
pub mod settings;
pub mod utils;

/// url not found handler
///
///

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

pub fn launch_async_server(address: &str, port: u16) -> std::io::Result<Server> {
    Ok(HttpServer::new(|| {
        App::new()
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
    })
    .bind((address, port))?
    .run())
}

#[actix_web::main]
/// it's launch the server in the given adrress and the given port
/// a call like this
///
/// # Example
/// ```
/// fn main() -> std::io:Result<()> {
///     let address = "127.0.0.1";
///     let port : u16 = 8090;
///     launch_server(address, port)
///     // it launch the server at 127.0.0.1:8090
/// }
/// ```
pub async fn launch_server(address: &str, port: u16) -> std::io::Result<()> {
    info!("launching mangadex-desktop-api on {}:{}", address, port);
    let habdle = launch_async_server(address, port)?.await;
    info!("closing mangadex-desktop-api on {}:{}", address, port);
    habdle
}

pub fn launch_async_server_default() -> std::io::Result<Server> {
    info!("launching server");
    let serve: server_options::ServerOptions = match server_options::ServerOptions::new() {
        Ok(data) => data,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };
    launch_async_server(serve.hostname.as_str(), serve.port)
}

/// Verify if the data dir and the settings are all there
/// if on of them are not defined or not found , it automatically create the dir corresponding to the error
pub fn verify_all_fs() -> std::io::Result<()> {
    match verify_settings_dir() {
        Ok(data) => {
            info!("{}", data);
        }
        Err(error) => {
            warn!("{}", error);
            warn!("Settings dir not found ");
            info!("Initializing...");
            match initialise_settings_dir() {
                Ok(data) => data,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ));
                }
            };
        }
    };
    info!("Initilized settings dir !");
    match verify_data_dir() {
        Ok(data) => {
            info!("{}", data);
        }
        Err(error) => {
            warn!("{}", error);
            warn!("Data dir not found \n");
            info!("\tInitializing...");
            match initialise_data_dir() {
                Ok(data) => data,
                Err(e) => {
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        e.to_string(),
                    ));
                }
            };
        }
    }
    load_history()?;
    Ok(())
}

/// It's launch the server with the given data in the settings/server_option.json
///
/// # Example

/// if we have a settings/server_option.json like this :
/// ```
/// {
///   "hostname" : "127.0.0.1",
///    "port" : 8090
/// }
/// ```
///
/// and we launch the function :
/// ```
/// fn main() -> std::io::Result<()> {
///     launch_server_default()
///     // it will launch the server at 127.0.0.1:8090
/// }
/// ```
pub fn launch_server_default() -> std::io::Result<()> {
    info!("launching server");
    let serve: server_options::ServerOptions = match server_options::ServerOptions::new() {
        Ok(data) => data,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ));
        }
    };
    launch_server(serve.hostname.as_str(), serve.port)
}
