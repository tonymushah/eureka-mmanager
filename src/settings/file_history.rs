use std::{io::{Write}};

use mangadex_api_types_rust::RelationshipType;
use serde::{Serialize, Deserialize};

use super::{files_dirs::DirsOptions};
use crate::r#static::history::init_static_history;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub struct HistoryEntry{
    id: uuid::Uuid,
    data_type: RelationshipType
}

impl HistoryEntry{
    pub fn new(id: uuid::Uuid, data_type: RelationshipType) -> HistoryEntry{
        return HistoryEntry { id: id, data_type: data_type };
    }
    pub fn get_id(&self) -> uuid::Uuid{
        return self.id;
    }
    pub fn get_data_type(&self) -> RelationshipType{
        return self.data_type;
    }
    pub fn set_id(&mut self, id: uuid::Uuid){
        self.id = id;
    }
    pub fn set_data_type(&mut self, data_type: RelationshipType){
        self.data_type = data_type;
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct History{
    history_list: Vec<uuid::Uuid>,
    data_type: RelationshipType
}

impl History{
    pub fn new(data_type: RelationshipType) -> History{
        return History { history_list: Vec::new(), data_type: data_type };
    }
    pub fn get_history_list_mut(&mut self)-> &mut Vec<uuid::Uuid>{
        return &mut (self.history_list);
    }
    pub fn get_history_list(&mut self)-> &Vec<uuid::Uuid>{
        return &(self.history_list);
    }
    pub fn get_data_type_mut(&mut self)-> &mut RelationshipType{
        return &mut (self.data_type);
    }
    pub fn get_data_type(&mut self)-> &RelationshipType{
        return &(self.data_type);
    }
    pub fn is_this_type(&self, to_use_rel: RelationshipType)-> bool{
        if self.data_type == to_use_rel {
            return true
        }else{
            return false;
        }
    }
    pub fn is_in(&self, id: uuid::Uuid)->bool{
        self.history_list.iter().any(|to_use| id.cmp(to_use).is_eq())
    }
    pub fn is_entry_in(&self, to_use: HistoryEntry)->Result<bool, std::io::Error>{
        if to_use.data_type == self.data_type {
            if self.is_in(to_use.id) {
                Ok(true)
            }else{
                Ok(false)
            }
        }else{
            Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "the relationship doesn't match"))
        }
    }
    pub fn add(&mut self, to_add: HistoryEntry)-> Result<(), std::io::Error>{
        let result = self.is_entry_in(to_add);
        if result? == false {
            self.history_list.push(to_add.id);
        }
        Ok(())
    }
    pub fn remove_uuid(&mut self, uuid : uuid::Uuid)-> Result<(), std::io::Error>{
        let result = self.is_in(uuid);
        if result == true{
            self.history_list.remove(match self.history_list.iter().position(|data| data.cmp(&uuid).is_eq()) {
                Some(data) => data,
                None => {
                    return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("the uuid {} is not found", uuid)))
                }
            });
        }else{
            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, format!("the uuid {} is not found", uuid)))
        }
        Ok(())
    }
    pub fn add_uuid(&mut self, to_add: uuid::Uuid) -> Result<(), std::io::Error>{
        let result = self.is_in(to_add);
        if result == false {
            self.history_list.push(to_add);
            return Ok(());
        }else {
            return Err(std::io::Error::new(std::io::ErrorKind::AlreadyExists, format!("the uuid {} is already there", to_add))); 
        }
    }
}

pub struct HistoryWFile{
    history : History,
    file : String
}

impl HistoryWFile{
    pub fn new(data_type: RelationshipType, file: String) -> HistoryWFile{
        return HistoryWFile{
            history : History::new(data_type),
            file : file
        };
    }
    pub fn get_history(&mut self) -> &mut History{
        return &mut (self.history); 
    }
    pub fn get_file(self) -> String{
        return self.file;
    }
    pub fn commit(&mut self) -> Result<(), std::io::Error>{
        let history_string_value = serde_json::to_string(&(self.history))?;
        let mut to_use_file = std::fs::File::options().create(true).truncate(true).write(true).open(&(self.file))?;
        to_use_file.write_all(history_string_value.as_bytes())?;
        Ok(())
    }
    pub fn rollback(&mut self) -> Result<(), std::io::Error>{
        let history_string_value = std::fs::read_to_string(&(self.file))?;
        let history : History = serde_json::from_str(&history_string_value)?;
        self.history = history;
        Ok(())
    }
    pub fn from_file(file: String) -> Result<HistoryWFile, std::io::Error>{
        let file_data: String = std::fs::read_to_string(&file)?;
        let history : History = serde_json::from_str(&file_data)?;
        Ok(HistoryWFile {
            history,
            file
        })
    }
}

pub fn init_history(relationship_type: RelationshipType, dir_options: &DirsOptions) -> Result<HistoryWFile, std::io::Error>{
    let path: String = dir_options.data_dir_add(format!("history/{}.json", serde_json::to_string(&relationship_type)?).replace("\"", "").as_str());
    let path_clone = path.clone();
    let history = match HistoryWFile::from_file(path) {
        Ok(data) => data,
        Err(_) => {
            HistoryWFile::new(relationship_type, path_clone)
        }
    };
    Ok(history)
}

pub fn init_history_dir() -> Result<(), std::io::Error>{
    let dir_options = match DirsOptions::new() {
        Ok(data) => data,
        Err(error) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()));
        }
    };
    let path: String = dir_options.data_dir_add(format!("history").as_str());
    std::fs::create_dir_all(path)?;
    Ok(())
}

pub fn load_history() -> Result<(), std::io::Error>{
    init_history_dir()?;
    init_static_history()?;
    Ok(())
}