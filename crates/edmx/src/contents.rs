use std::collections::HashMap;

use api_core::DirsOptions;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PChapterObject {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<String>,
    #[serde(default, rename = "data-saver", skip_serializing_if = "Vec::is_empty")]
    pub data_saver: Vec<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PMangaObject {
    pub covers: Vec<Uuid>,
    pub chapters: HashMap<Uuid, PChapterObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct PackageContents {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<DirsOptions>,
    pub data: HashMap<Uuid, PMangaObject>,
}
