use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind},
    path::{Path, PathBuf},
    sync::Arc,
};

use tokio::sync::{
    OwnedRwLockReadGuard, OwnedRwLockWriteGuard, RwLock, RwLockReadGuard, RwLockWriteGuard,
};

use mangadex_api_types_rust::RelationshipType;
use serde::Serialize;

use crate::{core::ManagerCoreResult, settings::files_dirs::DirsOptions, Error};

use self::traits::{AutoCommitRollbackInsert, AutoCommitRollbackRemove, Commitable, RollBackable};

use super::{HistoryBase, HistoryEntry, Insert, IsIn, Remove};

pub mod traits;

#[derive(Clone, Debug)]
pub struct HistoryWFile {
    history: Arc<RwLock<HistoryBase>>,
    file: PathBuf,
}

impl HistoryWFile {
    pub fn new<P: Into<PathBuf>>(data_type: RelationshipType, file: P) -> HistoryWFile {
        HistoryWFile {
            history: Arc::new(RwLock::new(HistoryBase::new(data_type))),
            file: file.into(),
        }
    }
    pub fn read_history(&self) -> Result<RwLockReadGuard<'_, HistoryBase>, std::io::Error> {
        self.history
            .try_read()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn write_history(&mut self) -> Result<RwLockWriteGuard<'_, HistoryBase>, std::io::Error> {
        self.history
            .try_write()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn owned_read_history(&self) -> Result<OwnedRwLockReadGuard<HistoryBase>, std::io::Error> {
        self.history
            .clone()
            .try_read_owned()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn owned_write_history(
        &mut self,
    ) -> Result<OwnedRwLockWriteGuard<HistoryBase>, std::io::Error> {
        self.history
            .clone()
            .try_write_owned()
            .map_err(|e| std::io::Error::new(ErrorKind::PermissionDenied, e.to_string()))
    }
    pub fn get_file(self) -> PathBuf {
        self.file
    }
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, std::io::Error> {
        let file_data = File::open(file.as_ref())?;
        let history: HistoryBase = serde_json::from_reader(BufReader::new(file_data))?;
        Ok(Self {
            history: history.into(),
            file: file.as_ref().to_path_buf(),
        })
    }
    pub fn init(
        relationship_type: RelationshipType,
        dir_options: &DirsOptions,
    ) -> Result<Self, std::io::Error> {
        let path = dir_options.history_add(
            format!("{}.json", serde_json::to_string(&relationship_type)?)
                .replace('\"', "")
                .as_str(),
        );
        let history = match Self::from_file(path.clone()) {
            Ok(data) => data,
            Err(_) => HistoryWFile::new(relationship_type, path),
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
        let mut serializer = serde_json::Serializer::new(BufWriter::new(to_use_file));
        history.serialize(&mut serializer)?;
        Ok(())
    }
}

impl RollBackable for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn rollback(&mut self) -> Self::Output {
        let history_string_value = std::fs::read_to_string(&(self.file))?;
        let mut history = self.write_history()?;
        *history = serde_json::from_str::<HistoryBase>(&history_string_value)?;
        Ok(())
    }
}

impl AutoCommitRollbackInsert<uuid::Uuid> for HistoryWFile {
    type Output = ManagerCoreResult<()>;

    fn insert(
        &mut self,
        input: uuid::Uuid,
    ) -> <Self as traits::AutoCommitRollbackInsert<uuid::Uuid>>::Output {
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

    fn remove(
        &mut self,
        input: uuid::Uuid,
    ) -> <Self as traits::AutoCommitRollbackRemove<uuid::Uuid>>::Output {
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

    fn insert(
        &mut self,
        input: HistoryEntry,
    ) -> <Self as traits::AutoCommitRollbackInsert<HistoryEntry>>::Output {
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

    fn remove(
        &mut self,
        input: HistoryEntry,
    ) -> <Self as traits::AutoCommitRollbackRemove<uuid::Uuid>>::Output {
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
    type Output = <HistoryBase as IsIn<HistoryEntry>>::Output;

    fn is_in(&self, to_use: HistoryEntry) -> Self::Output {
        self.read_history()?.is_in(to_use)
    }
}