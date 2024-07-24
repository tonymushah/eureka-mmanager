use api_core::DirsOptions;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PackageContentsOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub directories: Option<DirsOptions>,
    pub compressed_images: bool,
    pub compressed_metadata: bool,
}

impl From<DirsOptions> for PackageContentsOptions {
    fn from(value: DirsOptions) -> Self {
        Self {
            directories: Some(value),
            ..Default::default()
        }
    }
}
