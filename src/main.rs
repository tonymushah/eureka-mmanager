
/*
    [x] finish the server options api
    [x] implements those modifiction to the entire app
    [x] the app can edit his settings
*/

//use std::fs;

use std::collections::HashMap;

use mangadex_api_types::RelationshipType;
use mangadex_desktop_api2::{launch_server_default, verify_all_fs, settings::{file_history::{History, HistoryWFile, HistoryEntry}, init_static_history, get_history, insert_in_history, commit_rel, get_history_w_file_by_rel}};

macro_rules! print_historyFile {
    ($to_use:expr) => {
        $to_use.get_history().get_history_list().iter().for_each(|data| {
            println!("{}", data);
        });
    };
}

fn main() -> std::io::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply().unwrap();
    verify_all_fs()?;
    launch_server_default()?;
    //match init_static_history() {
    //    Ok(_) => (),
    //    Err(error) => {
    //        return Err(error);
    //    }
    //};
    //let choosed_rel = RelationshipType::Author;
    //let entry = HistoryEntry::new(uuid::Uuid::new_v4(), choosed_rel);
    //insert_in_history(&entry)?;
    //commit_rel(choosed_rel)?;
    //print_historyFile!(get_history_w_file_by_rel(choosed_rel)?);
    Ok(())
}