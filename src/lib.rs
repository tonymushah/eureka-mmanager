use crate::settings::file_history::load_history;
use crate::settings::verifications::{
        data::{initialise_data_dir, verify_data_dir}, 
        settings::{initialise_settings_dir, verify_settings_dir}
    };
use actix_web::dev::Server;
use log::{info, warn};
use settings::server_options;
mod r#core;

mod methods;
pub mod download;
pub mod settings;
pub mod utils;
pub mod server;
pub mod r#static;
#[cfg(feature = "feeds")]
pub mod feeds;
#[cfg(feature = "tauri")]
pub mod tauri;
/// url not found handler
///
///

pub use crate::server::launch_async_server;

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
/// ``` json
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
