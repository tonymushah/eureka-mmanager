use mangadex_api_types_rust::RelationshipType;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Hash, PartialEq, Eq)]
pub struct HistoryEntry {
    pub(crate) id: uuid::Uuid,
    pub(crate) data_type: RelationshipType,
}

impl HistoryEntry {
    pub fn new(id: uuid::Uuid, data_type: RelationshipType) -> HistoryEntry {
        HistoryEntry { id, data_type }
    }
    pub fn get_id(&self) -> uuid::Uuid {
        self.id
    }
    pub fn get_data_type(&self) -> RelationshipType {
        self.data_type
    }
    pub fn set_id(&mut self, id: uuid::Uuid) {
        self.id = id;
    }
    pub fn set_data_type(&mut self, data_type: RelationshipType) {
        self.data_type = data_type;
    }
}
