
/*
    [x] finish the server options api
    [x] implements those modifiction to the entire app
    [x] the app can edit his settings
*/

//use std::fs;

use mangadex_desktop_api2::{launch_server_default, verify_all_fs};


fn main() -> std::io::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply().unwrap();
    verify_all_fs()?;
    launch_server_default()?;
    Ok(())
}