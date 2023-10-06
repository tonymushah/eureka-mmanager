use std::fmt::Debug;

use futures::Future;
use mangadex_api_types_rust::RelationshipType;
use tokio::task::AbortHandle;

use crate::{
    core::ManagerCoreResult,
    settings::file_history::{
        history_w_file::traits::{
            NoLFAsyncAutoCommitRollbackInsert, NoLFAsyncAutoCommitRollbackRemove,
        },
        HistoryEntry, HistoryWFile, NoLFAsyncInsert, NoLFAsyncIsIn, NoLFAsyncRemove,
    },
};

#[async_trait::async_trait]
pub trait AccessHistory:
    NoLFAsyncInsert<HistoryEntry, Output = ManagerCoreResult<()>>
    + NoLFAsyncRemove<HistoryEntry, Output = ManagerCoreResult<()>>
    + NoLFAsyncAutoCommitRollbackInsert<HistoryEntry, Output = ManagerCoreResult<()>>
    + NoLFAsyncAutoCommitRollbackRemove<HistoryEntry, Output = ManagerCoreResult<()>>
    + NoLFAsyncIsIn<HistoryEntry, Output = ManagerCoreResult<bool>>
    + Send
{
    async fn init_history<'a>(
        &'a self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()>;
    async fn get_history_w_file_by_rel<'a>(
        &'a self,
        relationship_type: RelationshipType,
    ) -> std::io::Result<HistoryWFile>;
    async fn get_history_w_file_by_rel_or_init<'a>(
        &'a self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<HistoryWFile>;
    async fn commit_rel<'a>(
        &'a mut self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()>;
    async fn rollback_rel<'a>(
        &'a mut self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()>;
}

#[async_trait::async_trait]
pub trait AccessDownloadTasks: Send {
    async fn verify_limit(&self) -> bool;
    async fn spawn<F>(&mut self, task: F) -> ManagerCoreResult<AbortHandle>
    where
        F: Future<Output = ()> + Send + 'static;
    async fn lock_spawn<F>(&mut self, task: F) -> ManagerCoreResult<AbortHandle>
    where
        F: Future<Output = ()> + Send + 'static;
    async fn spawn_with_data<T>(&mut self, task: T) -> ManagerCoreResult<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + Debug + 'static;
    async fn lock_spawn_with_data<T>(&mut self, task: T) -> ManagerCoreResult<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static;
}
