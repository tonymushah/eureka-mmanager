use std::borrow::Cow;

use api_core::DirsOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PackageContentsOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directories: Option<DirsOptions>,
    pub zstd_compressed_images: bool,
    pub zstd_compressed_metadata: bool,
}

impl PackageContentsOptions {
    pub fn get_dirs(&self) -> Cow<'_, DirsOptions> {
        self.directories
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_default()
    }
}

impl From<DirsOptions> for PackageContentsOptions {
    fn from(value: DirsOptions) -> Self {
        Self {
            directories: Some(value),
            ..Default::default()
        }
    }
}
