use std::path::{Path, PathBuf};

use super::DirsOptions;

impl DirsOptions {
    pub fn mangas_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if self.mangas.is_absolute() || self.mangas.starts_with(&self.data_dir) {
            self.mangas.join(path)
        } else {
            self.data_dir_add(&self.mangas).join(path)
        }
    }
}
