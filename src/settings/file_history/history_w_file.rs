use std::{io::ErrorKind, sync::Arc};

use tokio::sync::{
    OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use mangadex_api_types_rust::RelationshipType;
use serde::Serialize;

use crate::{core::ManagerCoreResult, settings::files_dirs::DirsOptions, Error};

use self::traits::{AutoCommitRollbackInsert, AutoCommitRollbackRemove, Commitable, RollBackable};

use super::{History, HistoryEntry, Insert, IsIn, Remove};

pub mod traits;

#[derive(Clone, Debug)]
pub struct HistoryWFile {
    history: Arc<RwLock<History>>,
    file: String,
}

impl HistoryWFile {
    pub fn new(data_type: RelationshipType, file: String) -> HistoryWFile {
        HistoryWFile {
            history: Arc::new(RwLock::new(History::new(data_type))),
            file,
        }
    }
    pub fn read_history(&self) -> Result<RwLockReadGuard<'_, History>, std::io::Error> {
        self.history
            .try_read()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn write_history(&mut self) -> Result<RwLockWriteGuard<'_, History>, std::io::Error> {
        self.history
            .try_write()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn owned_read_history(&self) -> Result<OwnedRwLockReadGuard<History>, std::io::Error> {
        self.history
            .clone()
            .try_read_owned()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn owned_write_history(
        &mut self,
    ) -> Result<OwnedRwLockWriteGuard<History>, std::io::Error> {
        self.history
            .clone()
            .try_write_owned()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn get_file(self) -> String {
        self.file
    }
    pub fn from_file(file: String) -> Result<Self, std::io::Error> {
        let file_data: String = std::fs::read_to_string(file.clone())?;
        let history: History = serde_json::from_str(file_data.as_str())?;
        Ok(Self {
            history: history.into(),
            file,
        })
    }
    pub fn init(
        relationship_type: RelationshipType,
        dir_options: &DirsOptions,
    ) -> Result<Self, std::io::Error> {
        let path: String = dir_options.data_dir_add(
            format!(
                "history/{}.json",
                serde_json::to_string(&relationship_type)?
            )
            .replace('\"', "")
            .as_str(),
        );
        let history = match Self::from_file(path.clone()) {
            Ok(data) => data,
            Err(_) => HistoryWFile::new(relationship_type, path.clone()),
        };
        Ok(history)
    }
}

impl Insert<uuid::Uuid> for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn insert(&mut self, input: uuid::Uuid) -> Self::Output {
        self.write_history()?.insert(input)
    }
}

impl Insert<HistoryEntry> for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn insert(&mut self, input: HistoryEntry) -> Self::Output {
        self.write_history()?.insert(input)
    }
}

impl Remove<uuid::Uuid> for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn remove(&mut self, input: uuid::Uuid) -> Self::Output {
        self.write_history()?.remove(input)
    }
}

impl Remove<HistoryEntry> for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn remove(&mut self, input: HistoryEntry) -> Self::Output {
        self.write_history()?.remove(input)
    }
}

impl Commitable for HistoryWFile {
    type Output = Result<(), std::io::Error>;

    fn commit(&mut self) -> Self::Output {
        let to_use_file = std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&(self.file))?;
        let history = self.write_history()?;
        let mut serializer = serde_json::Serializer::new(to_use_file);
        history.serialize(&mut serializer)?;
        Ok(())
    }
}

impl RollBackable for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn rollback(&mut self) -> Self::Output {
        let history_string_value = std::fs::read_to_string(&(self.file))?;
        let mut history = self.write_history()?;
        *history = serde_json::from_str::<History>(&history_string_value)?;
        Ok(())
    }
}

impl AutoCommitRollbackInsert<uuid::Uuid> for HistoryWFile {
    type Output = ManagerCoreResult<()>;

    fn insert(&mut self, input : uuid::Uuid) -> <Self as crate::settings::file_history::history_w_file::traits::AutoCommitRollbackInsert<uuid::Uuid>>::Output{
        if let Err(error) = <Self as Insert<uuid::Uuid>>::insert(self, input) {
            if error.kind() == ErrorKind::AlreadyExists {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::Io(error))
            }
        } else if let Err(error) = self.commit() {
            self.rollback()?;
            Err(Error::RollBacked(error.to_string()))
        } else {
            Ok(())
        }
    }
}

impl AutoCommitRollbackRemove<uuid::Uuid> for HistoryWFile {
    type Output = ManagerCoreResult<()>;

    fn remove(&mut self, input : uuid::Uuid) -> <Self as crate::settings::file_history::history_w_file::traits::AutoCommitRollbackRemove<uuid::Uuid>>::Output{
        if let Err(error) = <Self as Remove<uuid::Uuid>>::remove(self, input) {
            if error.kind() == ErrorKind::NotFound {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::Io(error))
            }
        } else if let Err(error) = self.commit() {
            self.rollback()?;
            Err(Error::RollBacked(error.to_string()))
        } else {
            Ok(())
        }
    }
}

impl AutoCommitRollbackInsert<HistoryEntry> for HistoryWFile {
    type Output = ManagerCoreResult<()>;

    fn insert(&mut self, input : HistoryEntry) -> <Self as crate::settings::file_history::history_w_file::traits::AutoCommitRollbackInsert<HistoryEntry>>::Output{
        if let Err(error) = <Self as Insert<HistoryEntry>>::insert(self, input) {
            if error.kind() == ErrorKind::AlreadyExists {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::Io(error))
            }
        } else if let Err(error) = self.commit() {
            self.rollback()?;
            Err(Error::RollBacked(error.to_string()))
        } else {
            Ok(())
        }
    }
}

impl AutoCommitRollbackRemove<HistoryEntry> for HistoryWFile {
    type Output = ManagerCoreResult<()>;

    fn remove(&mut self, input : HistoryEntry) -> <Self as crate::settings::file_history::history_w_file::traits::AutoCommitRollbackRemove<uuid::Uuid>>::Output{
        if let Err(error) = <Self as Remove<HistoryEntry>>::remove(self, input) {
            if error.kind() == ErrorKind::NotFound {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::Io(error))
            }
        } else if let Err(error) = self.commit() {
            self.rollback()?;
            Err(Error::RollBacked(error.to_string()))
        } else {
            Ok(())
        }
    }
}

impl IsIn<uuid::Uuid> for HistoryWFile {
    type Output = ManagerCoreResult<bool>;

    fn is_in(&self, to_use: uuid::Uuid) -> Self::Output {
        Ok(self.read_history()?.is_in(to_use).is_some())
    }
}

impl IsIn<HistoryEntry> for HistoryWFile {
    type Output = <History as IsIn<HistoryEntry>>::Output;

    fn is_in(&self, to_use: HistoryEntry) -> Self::Output {
        self.read_history()?.is_in(to_use)
    }
}
