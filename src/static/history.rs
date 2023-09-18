use std::{collections::HashMap, ops::Deref, sync::Arc};

use mangadex_api_types_rust::RelationshipType;
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    core::{Error, ManagerCoreResult},
    settings::{
        file_history::{HistoryEntry, HistoryWFile, AsyncInsert, AsyncRemove, Insert, Remove, history_w_file::traits::{Commitable, RollBackable, AsyncCommitableWInput, AsyncRollBackableWInput, AutoCommitRollbackInsert, AutoCommitRollbackRemove, AsyncAutoCommitRollbackRemove, AsyncAutoCommitRollbackInsert, NoLFAsyncAutoCommitRollbackInsert, NoLFAsyncAutoCommitRollbackRemove}, AsyncIsIn, IsIn, NoLFAsyncRemove, NoLFAsyncInsert, NoLFAsyncIsIn},
        files_dirs::DirsOptions,
    },
};

//use self::file_history::History;

pub type InnerHistoryMap = HashMap<RelationshipType, HistoryWFile>;
#[derive(Default, Clone)]
pub struct HistoryMap(Arc<Mutex<InnerHistoryMap>>);

impl From<Arc<Mutex<InnerHistoryMap>>> for HistoryMap {
    fn from(value: Arc<Mutex<InnerHistoryMap>>) -> Self {
        Self(value)
    }
}

impl Deref for HistoryMap {
    type Target = Arc<Mutex<InnerHistoryMap>>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl HistoryMap {
    pub fn get_ref(&self) -> &Mutex<InnerHistoryMap> {
        self.0.as_ref()
    }
    pub fn into_inner(&self) -> Arc<Mutex<InnerHistoryMap>> {
        self.0.clone()
    }
    pub async fn get_history(&self) -> MutexGuard<'_, InnerHistoryMap> {
        self.lock().await
    }
    pub async fn init_history(
        &self,
        dir_option: &DirsOptions,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()> {
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::init_history(
            &mut history,
            dir_option,
            relationship_type,
        )
        .await
    }
    pub async fn get_history_w_file_by_rel(
        &self,
        relationship_type: RelationshipType,
    ) -> std::io::Result<HistoryWFile> {
        let history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel(
            &history,
            relationship_type,
        )
        .await
    }
    pub async fn get_history_w_file_by_rel_or_init(
        &self,
        relationship_type: RelationshipType,
        dir_options: &DirsOptions,
    ) -> ManagerCoreResult<HistoryWFile> {
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel_or_init(
            &mut history,
            relationship_type,
            dir_options,
        )
        .await
    }
    pub fn init_history_dir(dir_options: &DirsOptions) -> Result<(), std::io::Error> {
        let path: String = dir_options.data_dir_add("history".to_string().as_str());
        std::fs::create_dir_all(path)?;
        Ok(())
    }
    pub async fn init(
        dir_option: &DirsOptions,
        to_init: Option<Vec<RelationshipType>>,
    ) -> ManagerCoreResult<Self> {
        let instance = Self::default();
        if let Some(rels) = to_init {
            for rel in rels {
                instance.init_history(dir_option, rel).await?;
            }
        }
        Ok(instance)
    }
    pub async fn load_history(
        dir_options: &DirsOptions,
        to_init: Option<Vec<RelationshipType>>,
    ) -> ManagerCoreResult<Self> {
        Self::init_history_dir(dir_options)?;
        Self::init(dir_options, to_init).await
    }
}

#[async_trait::async_trait]
pub trait HistoryMapWithMutexGuardOnly {
    async fn init_history(
        history: &mut MutexGuard<'_, InnerHistoryMap>,
        dir_option: &DirsOptions,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()> {
        history.insert(
            relationship_type,
            HistoryWFile::init(relationship_type, dir_option)?,
        );
        Ok(())
    }
    async fn get_history_w_file_by_rel(
        history: &MutexGuard<'_, InnerHistoryMap>,
        relationship_type: RelationshipType,
    ) -> std::io::Result<HistoryWFile> {
        if let Some(data) = history.get(&relationship_type) {
            Ok(Clone::clone(data))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "value of {}",
                    serde_json::to_string(&relationship_type)?
                        .replace('\"', "")
                        .as_str()
                ),
            ))
        }
    }
    async fn get_history_w_file_by_rel_or_init(
        history: &mut MutexGuard<'_, InnerHistoryMap>,
        relationship_type: RelationshipType,
        dir_options: &DirsOptions,
    ) -> ManagerCoreResult<HistoryWFile> {
        let history_w_file = match Self::get_history_w_file_by_rel(history, relationship_type).await
        {
            Ok(data) => data,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::NotFound {
                    Self::init_history(history, dir_options, relationship_type).await?;
                    Self::get_history_w_file_by_rel(history, relationship_type).await?
                } else {
                    return Err(Error::Io(error));
                }
            }
        };
        Ok(history_w_file)
    }
    async fn insert_in_history(
        history: &mut MutexGuard<'_, InnerHistoryMap>,
        to_insert: &HistoryEntry,
        dir_options: &DirsOptions,
    ) -> ManagerCoreResult<()> {
        let mut history_w_file = Self::get_history_w_file_by_rel_or_init(
            history,
            to_insert.get_data_type(),
            dir_options,
        )
        .await?;
        <HistoryWFile as Insert<HistoryEntry>>::insert(&mut history_w_file, *to_insert)?;
        Ok(())
    }
    async fn remove_in_history(
        history: &mut MutexGuard<'_, InnerHistoryMap>,
        to_remove: &HistoryEntry,
        dir_options: &DirsOptions,
    ) -> ManagerCoreResult<()> {
        let mut history_w_file = Self::get_history_w_file_by_rel_or_init(
            history,
            to_remove.get_data_type(),
            dir_options,
        )
        .await?;
        <HistoryWFile as Remove<HistoryEntry>>::remove(&mut history_w_file, *to_remove)?;
        Ok(())
    }
    async fn commit_rel(
        history: &mut MutexGuard<'_, InnerHistoryMap>,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()> {
        let mut history_w_file =
            Self::get_history_w_file_by_rel(history, relationship_type).await?;
        history_w_file.commit()?;
        Ok(())
    }
    async fn rollback_rel(
        history: &mut MutexGuard<'_, InnerHistoryMap>,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()> {
        let mut history_w_file =
            Self::get_history_w_file_by_rel(history, relationship_type).await?;
        history_w_file.rollback()?;
        Ok(())
    }
}

impl HistoryMapWithMutexGuardOnly for HistoryMap {}

#[async_trait::async_trait]
impl<'a> AsyncInsert<'a, (HistoryEntry, &'a DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn insert(&'a mut self, input : (HistoryEntry, &'a DirsOptions)) -> Self::Output{
        let (to_insert, dir_options) = input;
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::insert_in_history(
            &mut history,
            &to_insert,
            dir_options,
        )
        .await
    }
}

#[async_trait::async_trait]
impl<'a> AsyncRemove<'a, (HistoryEntry, &'a DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn remove(&'a mut self, input : (HistoryEntry, &'a DirsOptions)) -> Self::Output{
        let (to_insert, dir_options) = input;
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::remove_in_history(
            &mut history,
            &to_insert,
            dir_options,
        )
        .await
    }
}

#[async_trait::async_trait]
impl<'a> AsyncCommitableWInput<'a, RelationshipType> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn commit(&'a mut self, input : RelationshipType) -> Self::Output {
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::commit_rel(&mut history, input).await
    }
}

#[async_trait::async_trait]
impl<'a> AsyncRollBackableWInput<'a, RelationshipType> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn rollback(&'a mut self, input : RelationshipType) -> Self::Output {
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::rollback_rel(&mut history, input).await
    }
}

#[async_trait::async_trait]
impl<'a> AsyncAutoCommitRollbackInsert<'a, (HistoryEntry, &'a DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn insert(&'a mut self, input : (HistoryEntry, &'a DirsOptions)) -> Self::Output{
        let (to_insert, dir_options) = input;
        let mut history = self.get_history().await;
        let mut history_w_file = <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel_or_init(
            &mut history,
            to_insert.get_data_type(),
            dir_options,
        )
        .await?;
        <HistoryWFile as AutoCommitRollbackInsert<HistoryEntry>>::insert(&mut history_w_file, to_insert)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<'a> AsyncAutoCommitRollbackRemove<'a, (HistoryEntry, &'a DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn remove(&'a mut self, input : (HistoryEntry, &'a DirsOptions)) -> Self::Output{
        let (to_remove, dir_options) = input;
        let mut history = self.get_history().await;
        let mut history_w_file = <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel_or_init(
            &mut history,
            to_remove.get_data_type(),
            dir_options,
        )
        .await?;
        <HistoryWFile as AutoCommitRollbackRemove<HistoryEntry>>::remove(&mut history_w_file, to_remove)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl<'a> AsyncIsIn<'a, (HistoryEntry, &'a DirsOptions)> for HistoryMap{
    type Output = ManagerCoreResult<bool>;
    async fn is_in(&'a self, to_use : (HistoryEntry, &'a DirsOptions)) -> Self::Output {
        let (to_is_in, dir_options) = to_use;
        let mut history = self.get_history().await;
        let history_w_file = <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel_or_init(
            &mut history,
            to_is_in.get_data_type(),
            dir_options,
        )
        .await?;
        Ok(<HistoryWFile as IsIn<HistoryEntry>>::is_in(&history_w_file, to_is_in)?)
    }
}



#[async_trait::async_trait]
impl NoLFAsyncInsert<(HistoryEntry, DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn insert(&mut self, input : (HistoryEntry, DirsOptions)) -> Self::Output{
        let (to_insert, dir_options) = input;
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::insert_in_history(
            &mut history,
            &to_insert,
            &dir_options,
        )
        .await
    }
}

#[async_trait::async_trait]
impl NoLFAsyncRemove<(HistoryEntry, DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn remove(&mut self, input : (HistoryEntry, DirsOptions)) -> Self::Output{
        let (to_insert, dir_options) = input;
        let mut history = self.get_history().await;
        <Self as HistoryMapWithMutexGuardOnly>::remove_in_history(
            &mut history,
            &to_insert,
            &dir_options,
        )
        .await
    }
}

#[async_trait::async_trait]
impl NoLFAsyncAutoCommitRollbackInsert<(HistoryEntry, DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn insert(&mut self, input : (HistoryEntry, DirsOptions)) -> Self::Output{
        let (to_insert, dir_options) = input;
        let mut history = self.get_history().await;
        let mut history_w_file = <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel_or_init(
            &mut history,
            to_insert.get_data_type(),
            &dir_options,
        )
        .await?;
        <HistoryWFile as AutoCommitRollbackInsert<HistoryEntry>>::insert(&mut history_w_file, to_insert)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl NoLFAsyncAutoCommitRollbackRemove<(HistoryEntry, DirsOptions)> for HistoryMap {
    type Output = ManagerCoreResult<()>;
    async fn remove(&mut self, input : (HistoryEntry, DirsOptions)) -> Self::Output{
        let (to_remove, dir_options) = input;
        let mut history = self.get_history().await;
        let mut history_w_file = <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel_or_init(
            &mut history,
            to_remove.get_data_type(),
            &dir_options,
        )
        .await?;
        <HistoryWFile as AutoCommitRollbackRemove<HistoryEntry>>::remove(&mut history_w_file, to_remove)?;
        Ok(())
    }
}

#[async_trait::async_trait]
impl NoLFAsyncIsIn<(HistoryEntry, DirsOptions)> for HistoryMap{
    type Output = ManagerCoreResult<bool>;
    async fn is_in(&self, to_use : (HistoryEntry, DirsOptions)) -> Self::Output {
        let (to_is_in, dir_options) = to_use;
        let mut history = self.get_history().await;
        let history_w_file = <Self as HistoryMapWithMutexGuardOnly>::get_history_w_file_by_rel_or_init(
            &mut history,
            to_is_in.get_data_type(),
            &dir_options,
        )
        .await?;
        Ok(<HistoryWFile as IsIn<HistoryEntry>>::is_in(&history_w_file, to_is_in)?)
    }
}