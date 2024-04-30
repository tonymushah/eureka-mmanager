use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use actix::{Actor, Context};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod messages;

use crate::{core::ManagerCoreResult, DirsOptionsVerificationError};

#[derive(Deserialize, Serialize, Clone)]
pub struct DirsOptions {
    pub data_dir: PathBuf,
    pub chapters: PathBuf,
    pub mangas: PathBuf,
    pub covers: PathBuf,
    #[serde(default)]
    pub init_dirs_if_not_exists: Option<bool>,
}

impl DirsOptions {
    pub fn load_from_path(path: &Path) -> ManagerCoreResult<DirsOptions> {
        let file = File::open(path)?;
        let instance: DirsOptions = serde_json::from_reader(BufReader::new(file))?;
        Ok(instance)
    }
    pub fn new_from_data_dir<P: AsRef<Path>>(data_dir: P) -> DirsOptions {
        let data_dir = data_dir.as_ref().to_path_buf();
        DirsOptions {
            chapters: data_dir.join("chapters"),
            mangas: data_dir.join("mangas"),
            covers: data_dir.join("covers"),
            init_dirs_if_not_exists: Some(true),
            data_dir,
        }
    }
    pub fn data_dir_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.data_dir.join(path)
    }
    pub fn chapters_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if self.chapters.is_absolute() || self.chapters.starts_with(&self.data_dir) {
            self.chapters.join(path)
        } else {
            self.data_dir_add(&self.chapters).join(path)
        }
    }
    pub fn mangas_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if self.mangas.is_absolute() || self.mangas.starts_with(&self.data_dir) {
            self.mangas.join(path)
        } else {
            self.data_dir_add(&self.mangas).join(path)
        }
    }
    pub fn covers_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if self.covers.is_absolute() || self.covers.starts_with(&self.data_dir) {
            self.covers.join(path)
        } else {
            self.data_dir_add(&self.covers).join(path)
        }
    }
    pub fn cover_images_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.covers_add("images").join(path)
    }
    pub fn chapters_id_add(&self, id: Uuid) -> PathBuf {
        let res = self.chapters_add(id.to_string());
        if self.init_dirs_if_not_exists.unwrap_or(true) {
            let _ = std::fs::create_dir_all(&res);
        }
        res
    }
    pub fn chapters_id_data_add(&self, id: Uuid) -> PathBuf {
        let res = self.chapters_id_add(id).join("data");
        if self.init_dirs_if_not_exists.unwrap_or(true) {
            let _ = std::fs::create_dir_all(&res);
        }
        res
    }
    pub fn chapters_id_data_saver_add(&self, id: Uuid) -> PathBuf {
        let res = self.chapters_id_add(id).join("data-saver");
        if self.init_dirs_if_not_exists.unwrap_or(true) {
            let _ = std::fs::create_dir_all(&res);
        }
        res
    }
    pub fn history_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.data_dir_add("history").join(path)
    }
    pub fn init_dirs(&self) -> ManagerCoreResult<()> {
        std::fs::create_dir_all(self.data_dir_add(""))?;
        std::fs::create_dir_all(self.history_add(""))?;
        std::fs::create_dir_all(self.chapters_add(""))?;
        std::fs::create_dir_all(self.covers_add(""))?;
        std::fs::create_dir_all(self.mangas_add(""))?;
        std::fs::create_dir_all(self.cover_images_add(""))?;
        Ok(())
    }
    pub fn verify(&self) -> Result<(), DirsOptionsVerificationError> {
        if !self.data_dir.exists() {
            return Err(DirsOptionsVerificationError::DataRoot);
        }
        if !self.history_add("").exists() {
            return Err(DirsOptionsVerificationError::History);
        }
        if !self.chapters.exists() {
            return Err(DirsOptionsVerificationError::Chapters);
        }
        if !self.covers.exists() {
            return Err(DirsOptionsVerificationError::Covers);
        }
        if !self.cover_images_add("").exists() {
            return Err(DirsOptionsVerificationError::CoverImages);
        }
        if !self.mangas.exists() {
            return Err(DirsOptionsVerificationError::Mangas);
        }
        Ok(())
    }
    pub fn verify_and_init(&self) -> ManagerCoreResult<()> {
        if let Ok(()) = self.verify() {
            Ok(())
        } else {
            self.init_dirs()?;
            Ok(())
        }
    }
}

impl Actor for DirsOptions {
    type Context = Context<Self>;
}
