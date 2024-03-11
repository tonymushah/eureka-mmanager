use uuid::Uuid;

use crate::{
    server::traits::AccessHistory,
    settings::file_history::{
        history_w_file::traits::NoLFAsyncAutoCommitRollbackInsert, HistoryEntry,
    },
    ManagerCoreResult,
};

use super::ChapterDownload;

impl ChapterDownload {
    pub async fn start_transation<'a, H>(
        &self,
        history: &'a mut H,
    ) -> ManagerCoreResult<HistoryEntry>
    where
        H: AccessHistory,
    {
        let chapter_id = Uuid::parse_str(self.chapter_id.to_string().as_str())?;
        let history_entry = HistoryEntry::new(
            chapter_id,
            mangadex_api_types_rust::RelationshipType::Chapter,
        );
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackInsert<HistoryEntry>>::insert(
            history,
            history_entry,
        )
        .await?;
        Ok(history_entry)
    }
}
