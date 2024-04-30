use std::path::{Path, PathBuf};
use uuid::Uuid;

use super::DirsOptions;

impl DirsOptions {
    pub fn chapters_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if self.chapters.is_absolute() || self.chapters.starts_with(&self.data_dir) {
            self.chapters.join(path)
        } else {
            self.data_dir_add(&self.chapters).join(path)
        }
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
}
