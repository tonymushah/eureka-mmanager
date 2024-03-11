use std::{fs::File, io::BufReader, path::Path};

use serde::{Deserialize, Serialize};

use crate::core::ManagerCoreResult;

#[derive(Deserialize, Serialize, Clone)]
pub struct DirsOptions {
    pub data_dir: String,
    pub chapters: String,
    pub mangas: String,
    pub covers: String,
}

impl DirsOptions {
    pub fn new_(path: &str) -> ManagerCoreResult<DirsOptions> {
        let file = File::open(path)?;
        let instance: DirsOptions = serde_json::from_reader(BufReader::new(file))?;
        Ok(instance)
    }
    pub fn new() -> ManagerCoreResult<DirsOptions> {
        DirsOptions::new_("./settings/files-dirs.json")
    }
    pub fn data_dir_add(&self, path: &str) -> String {
        format!("{}/{}", self.data_dir, path)
    }
    pub fn chapters_add(&self, path: &str) -> String {
        let chapters_path = self.chapters.as_str();
        if Path::new(chapters_path).is_absolute() {
            return format!("{chapters_path}/{path}");
        }
        let chapters_path_defpath = self.data_dir_add(chapters_path);
        format!("{}/{}", chapters_path_defpath, path)
    }
    pub fn mangas_add(&self, path: &str) -> String {
        let mangas_path = self.mangas.as_str();
        if Path::new(mangas_path).is_absolute() {
            return format!("{mangas_path}/{path}");
        }
        let mangas_path_defpath = self.data_dir_add(mangas_path);
        format!("{}/{}", mangas_path_defpath, path)
    }
    pub fn covers_add(&self, path: &str) -> String {
        let covers_path = self.covers.as_str();
        if Path::new(covers_path).is_absolute() {
            return format!("{covers_path}/{path}");
        }
        let covers_path_defpath = self.data_dir_add(covers_path);
        format!("{}/{}", covers_path_defpath, path)
    }

    /*
    pub fn _data_dir_add<'a>(&'a self, path: &'a str) -> &'a str {
        (&self.data_dir).join("/").join(path)
        //&format!("{}/{}", self.data_dir, path)
    }
    pub fn _chapters_add<'a>(&'a self, path: &'a str) -> &'a str {
        let chapters_path = self.chapters.as_str();
        let chapters_path_defpath = self._data_dir_add(chapters_path);
        &format!("{}/{}", chapters_path_defpath, path)
    }
    pub fn _mangas_add<'a>(&'a self, path: &'a str) -> &'a str {
        let mangas_path = self.mangas.as_str();
        let mangas_path_defpath = self._data_dir_add(mangas_path);
        &format!("{}/{}", mangas_path_defpath, path)
    }
    pub fn _covers_add<'a>(&'a self, path: &'a str) -> &'a str {
        let covers_path = self.covers.as_str();
        let covers_path_defpath = self._data_dir_add(covers_path);
        &format!("{}/{}", covers_path_defpath, path)
    }
    */
}
