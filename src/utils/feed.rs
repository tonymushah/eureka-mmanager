use std::cmp::Ordering;

use mangadex_api_schema::{ApiObject, v5::ChapterAttributes};
use serde::{Serialize, Deserialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct ChapterFeed(ApiObject<ChapterAttributes>);

impl ChapterFeed {
    pub fn new(value : ApiObject<ChapterAttributes>) -> ChapterFeed {
        ChapterFeed(value)
    }
}

impl Eq for ChapterFeed {}

impl PartialOrd for ChapterFeed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0
            .attributes
            .readable_at
            .as_ref()
            .partial_cmp(other.0.attributes.readable_at.as_ref())
    }
}

impl AsRef<ApiObject<ChapterAttributes>> for ChapterFeed {
    fn as_ref(&self) -> &ApiObject<ChapterAttributes> {
        &self.0
    }
}

impl PartialEq for ChapterFeed {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Ord for ChapterFeed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0.id.cmp(&other.0.id) == Ordering::Equal {
            return Ordering::Equal;
        }
        self.0
            .attributes
            .readable_at
            .as_ref()
            .cmp(other.0.attributes.readable_at.as_ref())
    }
}
