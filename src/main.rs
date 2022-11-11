
/*
    [x] finish the server options api
    [x] implements those modifiction to the entire app
    [x] the app can edit his settings
*/

//use std::fs;

use mangadex_desktop_api2::{launch_server_default, verify_all_fs};

fn main() -> std::io::Result<()> {
    //fs::remove_dir_all("./data/chapters/f65d40d7-b1e6-4cea-8764-6b3018fc159f")?;

    verify_all_fs()?;
    launch_server_default()?;
    Ok(())
}
