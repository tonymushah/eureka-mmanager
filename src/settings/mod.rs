use std::{collections::HashMap, io::Write, sync::Mutex};

use mangadex_api_types::RelationshipType;

use self::{
    file_history::{init_history, HistoryEntry, HistoryWFile},
    files_dirs::DirsOptions,
};

//use self::file_history::History;

pub mod file_history;
pub mod files_dirs;
pub mod server_options;

static mut HISTORY: once_cell::sync::OnceCell<Mutex<HashMap<RelationshipType, HistoryWFile>>> =
    once_cell::sync::OnceCell::new();

pub fn init_static_history() -> Result<(), std::io::Error> {
    let thread = std::thread::spawn(|| unsafe {
        let dir_options: DirsOptions = match DirsOptions::new() {
            Ok(data) => data,
            Err(_) => {
                panic!("Error on loading history");
            }
        };
        let mut instance: HashMap<RelationshipType, HistoryWFile> = HashMap::new();
        instance.insert(
            RelationshipType::Manga,
            match init_history(RelationshipType::Manga, &dir_options) {
                Ok(data) => data,
                Err(_) => {
                    panic!("Error on loading manga history");
                }
            },
        );
        match HISTORY.get() {
            None => {
                match HISTORY.set(Mutex::new(instance)) {
                    Ok(a) => a,
                    Err(_) => std::panic::panic_any(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Error on initing static history",
                    )),
                };
            },
            Some(_) => ()
        }
    })
    .join();
    match thread {
        Ok(_) => (),
        Err(_) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Error on initing static history",
            ));
        }
    }
    Ok(())
}

pub fn get_history() -> Result<&'static mut HashMap<RelationshipType, HistoryWFile>, std::io::Error>
{
    let to_return: &mut HashMap<RelationshipType, HistoryWFile>;
    unsafe {
        let data = match HISTORY.get_mut() {
            Some(data) => data,
            None => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "the history is not initialized",
                ))
            }
        };
        to_return = match data.get_mut() {
            Ok(data) => data,
            Err(error) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    error.to_string(),
                ))
            }
        }
    }
    Ok(to_return)
}

/// it's initialise the dir and all json settings
pub fn initialise_settings_dir() -> anyhow::Result<()> {
    std::fs::create_dir_all("./settings")?;

    // initialise the files-dirs.json file
    let mut file_dir = std::fs::File::create("./settings/files-dirs.json")?;
    // default config
    let file_dir_content = serde_json::json!({
        "data_dir" : "data",
        "chapters" : "chapters",
        "mangas" : "mangas",
        "covers" : "covers"
    });
    file_dir.write_all(file_dir_content.to_string().as_bytes())?;

    // initialise the server-option
    let mut server_option = std::fs::File::create("./settings/server-options.json")?;
    // default config
    let server_option_content = serde_json::json!({
        "hostname" : "127.0.0.1",
        "port" : 8145
    });
    server_option.write_all(server_option_content.to_string().as_bytes())?;

    Ok(())
}

pub fn verify_settings_dir() -> anyhow::Result<String, String> {
    if std::path::Path::new("./settings").exists() == false {
        return Err("the dir settings doesn't exist".to_string());
    }
    if std::path::Path::new("./settings/files-dirs.json").exists() == false {
        return Err("the files-dirs.json in the settings dir doesn't exist".to_string());
    }
    if std::path::Path::new("./settings/server-options.json").exists() == false {
        return Err("the server-options.json in the settings dir doesn't exist".to_string());
    }
    Ok("the dir settings is operationnal".to_string())
}

pub fn initialise_data_dir() -> anyhow::Result<()> {
    let dirs_options: files_dirs::DirsOptions = match files_dirs::DirsOptions::new() {
        Ok(data) => data,
        Err(_) => {
            return Err(anyhow::Error::msg("can't load the dir options api"));
        }
    };
    std::fs::create_dir_all(dirs_options.data_dir_add(""))?;
    std::fs::create_dir_all(dirs_options.chapters_add(""))?;
    std::fs::create_dir_all(dirs_options.covers_add(""))?;
    std::fs::create_dir_all(dirs_options.mangas_add(""))?;
    std::fs::create_dir_all(dirs_options.covers_add("lists"))?;
    std::fs::create_dir_all(dirs_options.covers_add("images"))?;
    Ok(())
}

pub fn verify_data_dir() -> anyhow::Result<String, String> {
    let dirs_options: files_dirs::DirsOptions = match files_dirs::DirsOptions::new() {
        Ok(data) => data,
        Err(_) => {
            return Err("can't load the file dir api".into());
        }
    };
    if std::path::Path::new(dirs_options.data_dir_add("").as_str()).exists() == false {
        return Err("the data dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options.chapters_add("").as_str()).exists() == false {
        return Err("the chapters dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options.covers_add("").as_str()).exists() == false {
        return Err("the covers dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options.mangas_add("").as_str()).exists() == false {
        return Err("the mangas dir doesn't exist".to_string());
    }
    Ok("the data dir is operational".to_string())
}

pub fn get_history_w_file_by_rel(
    relationship_type: RelationshipType,
) -> Result<&'static mut HistoryWFile, std::io::Error> {
    let history = get_history()?;
    match history.get_mut(&(relationship_type)) {
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "value of {}",
                    serde_json::to_string(&relationship_type)?
                        .replace("\"", "")
                        .as_str()
                ),
            ));
        }
        Some(data) => return Ok(data),
    };
}

pub fn insert_in_history(to_insert: &HistoryEntry) -> Result<(), std::io::Error> {
    let history = get_history()?;
    let dir_options: DirsOptions = match DirsOptions::new() {
        Ok(data) => data,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            ))
        }
    };
    let history_w_file = match get_history_w_file_by_rel(to_insert.get_data_type()) {
        Ok(data) => data,
        Err(error) => {
            let to_use;
            if error.kind() == std::io::ErrorKind::NotFound {
                history.insert(
                    to_insert.get_data_type(),
                    init_history(to_insert.get_data_type(), &dir_options)?,
                );
                match get_history_w_file_by_rel(to_insert.get_data_type()) {
                    Ok(data) => {
                        to_use = data;
                    }
                    Err(err) => return Err(err),
                };
            } else {
                return Err(error);
            }
            to_use
        }
    };
    history_w_file.get_history().add_uuid(to_insert.get_id())?;
    Ok(())
}

pub fn remove_in_history(to_remove: &HistoryEntry) -> Result<(), std::io::Error> {
    let history_w_file = match get_history_w_file_by_rel(to_remove.get_data_type()) {
        Ok(data) => data,
        Err(error) => {
            return Err(error);
        }
    };
    history_w_file
        .get_history()
        .remove_uuid(to_remove.get_id())?;
    Ok(())
}

pub fn commit_rel(relationship_type: RelationshipType) -> Result<(), std::io::Error> {
    let history_w_file = match get_history_w_file_by_rel(relationship_type) {
        Ok(data) => data,
        Err(error) => {
            return Err(error);
        }
    };
    history_w_file.commit()?;
    Ok(())
}

pub fn rollback_rel(relationship_type: RelationshipType) -> Result<(), std::io::Error> {
    let history_w_file = match get_history_w_file_by_rel(relationship_type) {
        Ok(data) => data,
        Err(error) => {
            return Err(error);
        }
    };
    history_w_file.rollback()?;
    Ok(())
}
