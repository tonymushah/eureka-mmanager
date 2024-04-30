use std::path::{Path, PathBuf};

use super::DirsOptions;

impl DirsOptions {
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
}
