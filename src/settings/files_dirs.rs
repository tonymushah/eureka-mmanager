use serde::{Deserialize, Serialize};

use crate::core::ManagerCoreResult;

#[derive(Deserialize, Serialize, Clone)]
pub struct DirsOptions{
    data_dir : String,
    chapters : String,
    mangas : String,
    covers : String
}

impl DirsOptions{
    pub fn new_() -> ManagerCoreResult<DirsOptions>{
        let instance : DirsOptions = serde_json::from_str(std::fs::read_to_string("./settings/files-dirs.json")?.as_str())?;
        Ok(instance)
    }
    pub fn new() -> ManagerCoreResult<DirsOptions>{
        let instance : DirsOptions = serde_json::from_str(std::fs::read_to_string("./settings/files-dirs.json")?.as_str())?;
        ManagerCoreResult::Ok(instance)
    }
    pub fn data_dir_add(&self, path: &str) -> String{
        format!("{}/{}", self.data_dir, path)
    }
    pub fn chapters_add(&self, path: &str) -> String {
        let chapters_path = self.chapters.as_str();
        let chapters_path_defpath = self.data_dir_add(chapters_path);
        format!("{}/{}", chapters_path_defpath, path)
    }
    pub fn mangas_add(&self, path: &str) -> String {
        let mangas_path = self.mangas.as_str();
        let mangas_path_defpath = self.data_dir_add(mangas_path);
        format!("{}/{}", mangas_path_defpath, path)
    }
    pub fn covers_add(&self, path: &str) -> String {
        let covers_path = self.covers.as_str();
        let covers_path_defpath = self.data_dir_add(covers_path);
        format!("{}/{}", covers_path_defpath, path)
    }
}