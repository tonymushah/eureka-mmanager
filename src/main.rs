use mangadex_desktop_api2::{launch_server_default, verify_all_fs};
//use mangadex_api::MangaDexClient;
#[cfg(feature = "unix-socket-support")]
use mangadex_desktop_api2::server::launch_async_server_with_unix_socket;

#[cfg(feature = "unix-socket-support")]
use std::os::unix::net::UnixListener;

#[cfg(feature = "unix-socket-support")]
#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
    verify_all_fs()?;
    if cfg!(feature = "unix-socket-support") {
        let unix_listener = UnixListener::bind("./socket/mangadex")?;
        launch_async_server_with_unix_socket(unix_listener)?.await?;
    }
    anyhow::Ok(())
}

#[cfg(not(feature = "unix-socket-support"))]
#[tokio::main]
async fn main() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply()
        .unwrap();
    verify_all_fs()?;
    launch_server_default().await?;
    anyhow::Ok(())
}
