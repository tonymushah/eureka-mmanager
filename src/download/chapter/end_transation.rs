use crate::{
    server::traits::AccessHistory,
    settings::file_history::{
        history_w_file::traits::NoLFAsyncAutoCommitRollbackRemove, HistoryEntry,
    },
    ManagerCoreResult,
};

use super::ChapterDownload;

impl ChapterDownload {
    pub async fn end_transation<'a, H>(
        &'a self,
        entry: HistoryEntry,
        history: &'a mut H,
    ) -> ManagerCoreResult<()>
    where
        H: AccessHistory,
    {
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackRemove<HistoryEntry>>::remove(
            history, entry,
        )
        .await?;
        Ok(())
    }
}
