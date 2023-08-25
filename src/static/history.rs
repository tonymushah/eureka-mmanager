use std::{
    collections::HashMap,
    ops::Deref,
    sync::Arc,
};

use mangadex_api_types_rust::RelationshipType;
use tokio::sync::{Mutex, MutexGuard};

use crate::{
    core::{Error, ManagerCoreResult},
    settings::{
        file_history::{HistoryEntry, HistoryWFile},
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
        history.insert(
            relationship_type,
            HistoryWFile::init(relationship_type, dir_option)?,
        );
        Ok(())
    }
    pub async fn get_history_w_file_by_rel(
        &self,
        relationship_type: RelationshipType,
    ) -> std::io::Result<HistoryWFile> {
        let history = self.get_history().await;
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
    pub async fn get_history_w_file_by_rel_or_init(
        &self,
        relationship_type: RelationshipType,
        dir_options: &DirsOptions,
    ) -> ManagerCoreResult<HistoryWFile> {
        let mut history = self.get_history().await;
        let history_w_file = match self.get_history_w_file_by_rel(relationship_type).await {
            Ok(data) => data,
            Err(error) => {
                if error.kind() == std::io::ErrorKind::NotFound {
                    history.insert(
                        relationship_type,
                        HistoryWFile::init(relationship_type, &dir_options)?,
                    );
                    self.get_history_w_file_by_rel(relationship_type).await?
                } else {
                    return Err(Error::Io(error));
                }
            }
        };
        Ok(history_w_file)
    }
    pub async fn insert_in_history(
        &self,
        to_insert: &HistoryEntry,
        dir_options: &DirsOptions,
    ) -> ManagerCoreResult<()> {
        let mut history_w_file = self
            .get_history_w_file_by_rel_or_init(to_insert.get_data_type(), dir_options)
            .await?;
        history_w_file.get_history().add_uuid(to_insert.get_id())?;
        Ok(())
    }
    pub async fn remove_in_history(
        &self,
        to_remove: &HistoryEntry,
        dir_options: &DirsOptions,
    ) -> ManagerCoreResult<()> {
        let mut history_w_file = self
            .get_history_w_file_by_rel_or_init(to_remove.get_data_type(), dir_options)
            .await?;
        history_w_file
            .get_history()
            .remove_uuid(to_remove.get_id())?;
        Ok(())
    }
    pub async fn commit_rel(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()> {
        let mut history_w_file = self.get_history_w_file_by_rel(relationship_type).await?;
        history_w_file.commit()?;
        Ok(())
    }
    pub async fn rollback_rel(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()> {
        let mut history_w_file = self.get_history_w_file_by_rel(relationship_type).await?;
        history_w_file.rollback()?;
        Ok(())
    }
    pub fn init_history_dir(dir_options: &DirsOptions) -> Result<(), std::io::Error> {
        let path: String = dir_options.data_dir_add("history".to_string().as_str());
        std::fs::create_dir_all(path)?;
        Ok(())
    }
    pub async fn init(dir_option: &DirsOptions, to_init : Option<Vec<RelationshipType>>) -> ManagerCoreResult<Self>{
        let instance = Self::default();
        if let Some(rels) = to_init {
            for rel in rels {
                instance.init_history(dir_option, rel).await?;
            }
        }
        Ok(instance)
    }
    pub async fn load_history(dir_options: &DirsOptions, to_init : Option<Vec<RelationshipType>>) -> ManagerCoreResult<Self> {
        Self::init_history_dir(dir_options)?;
        Self::init(dir_options, to_init).await
    }
}
