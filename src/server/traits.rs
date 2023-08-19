use mangadex_api_types_rust::RelationshipType;

use crate::{
    core::ManagerCoreResult,
    settings::file_history::{HistoryEntry, HistoryWFile},
};

#[async_trait::async_trait]
pub trait AccessHistory {
    async fn init_history(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()>;
    async fn get_history_w_file_by_rel(
        &self,
        relationship_type: RelationshipType,
    ) -> std::io::Result<HistoryWFile>;
    async fn get_history_w_file_by_rel_or_init(
        &self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<HistoryWFile>;
    async fn insert_in_history(&self, to_insert: &HistoryEntry) -> ManagerCoreResult<()>;
    async fn remove_in_history(&self, to_remove: &HistoryEntry) -> ManagerCoreResult<()>;
    async fn commit_rel(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()>;
    async fn rollback_rel(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()>;
}
