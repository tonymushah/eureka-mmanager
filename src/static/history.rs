use std::{collections::HashMap, sync::Mutex};

use mangadex_api_types_rust::RelationshipType;

use crate::settings::{
    file_history::{init_history, HistoryEntry, HistoryWFile},
    files_dirs::DirsOptions,
};

//use self::file_history::History;

static mut HISTORY: once_cell::sync::OnceCell<Mutex<HashMap<RelationshipType, HistoryWFile>>> =
    once_cell::sync::OnceCell::new();

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
pub fn get_history_w_file_by_rel_or_init(relationship_type: RelationshipType) -> Result<&'static mut HistoryWFile, std::io::Error>{
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
    let history_w_file = match get_history_w_file_by_rel(relationship_type) {
        Ok(data) => data,
        Err(error) => {
            let to_use;
            if error.kind() == std::io::ErrorKind::NotFound {
                history.insert(
                    relationship_type,
                    init_history(relationship_type, &dir_options)?,
                );
                match get_history_w_file_by_rel(relationship_type) {
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
    Ok(history_w_file)
}

pub fn insert_in_history(to_insert: &HistoryEntry) -> Result<(), std::io::Error> {
    let history_w_file = get_history_w_file_by_rel_or_init(to_insert.get_data_type())?;
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

pub fn init_static_history() -> Result<(), std::io::Error> {
    let thread = std::thread::spawn(|| -> Result<(), std::io::Error> { unsafe {
        let dir_options: DirsOptions = match DirsOptions::new() {
            Ok(data) => data,
            Err(error) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()));
            }
        };
        let mut instance: HashMap<RelationshipType, HistoryWFile> = HashMap::new();
        instance.insert(
            RelationshipType::Manga,
            match init_history(RelationshipType::Manga, &dir_options) {
                Ok(data) => data,
                Err(error) => {
                    return Err(error);
                }
            },
        );
        match HISTORY.get() {
            None => {
                match HISTORY.set(Mutex::new(instance)) {
                    Ok(a) => return Ok(a),
                    Err(_) => {
                        return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Error on initing static history"
                    ))},
                };
            },
            Some(_) => ()
        };
        Ok(())
    }})
    .join();
    match thread {
        Ok(getted) => getted?,
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
