use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::{Path, PathBuf},
};

use actix::Addr;
use mangadex_api_types_rust::RelationshipType;
use serde::Serialize;

use crate::{core::ManagerCoreResult, files_dirs::DirsOptions, Error, JoinHistoryMessage};

use self::traits::{AutoCommitRollbackInsert, AutoCommitRollbackRemove, Commitable, RollBackable};

use super::{base::HBResult, HistoryBase, HistoryBaseError, HistoryEntry, Insert, IsIn, Remove};

pub mod traits;

#[derive(Clone, Debug)]
pub struct HistoryWFile {
    history: HistoryBase,
    file: PathBuf,
}

impl HistoryWFile {
    pub fn new<P: Into<PathBuf>>(data_type: RelationshipType, file: P) -> HistoryWFile {
        HistoryWFile {
            history: HistoryBase::new(data_type),
            file: file.into(),
        }
    }
    pub fn get_history(&self) -> &HistoryBase {
        &self.history
    }
    pub fn get_file(self) -> PathBuf {
        self.file
    }
    pub fn from_file<P: AsRef<Path>>(file: P) -> Result<Self, std::io::Error> {
        let file_data = File::open(file.as_ref())?;
        let history: HistoryBase = serde_json::from_reader(BufReader::new(file_data))?;
        Ok(Self {
            history,
            file: file.as_ref().to_path_buf(),
        })
    }
    pub async fn init(
        relationship_type: RelationshipType,
        dir_options: Addr<DirsOptions>,
    ) -> ManagerCoreResult<Self> {
        let path = dir_options
            .send(JoinHistoryMessage(
                format!("{}.json", serde_json::to_string(&relationship_type)?).replace('\"', ""),
            ))
            .await?;
        let history = match Self::from_file(path.clone()) {
            Ok(data) => data,
            Err(_) => HistoryWFile::new(relationship_type, path),
        };
        Ok(history)
    }
}

impl Insert<uuid::Uuid> for HistoryWFile {
    type Output = HBResult<()>;
    fn insert(&mut self, input: uuid::Uuid) -> Self::Output {
        self.history.insert(input)
    }
}

impl Insert<HistoryEntry> for HistoryWFile {
    type Output = HBResult<()>;
    fn insert(&mut self, input: HistoryEntry) -> Self::Output {
        self.history.insert(input)
    }
}

impl Remove<uuid::Uuid> for HistoryWFile {
    type Output = HBResult<()>;
    fn remove(&mut self, input: uuid::Uuid) -> Self::Output {
        self.history.remove(input)
    }
}

impl Remove<HistoryEntry> for HistoryWFile {
    type Output = HBResult<()>;
    fn remove(&mut self, input: HistoryEntry) -> Self::Output {
        self.history.remove(input)
    }
}

impl Commitable for HistoryWFile {
    type Output = ManagerCoreResult<()>;

    fn commit(&self) -> Self::Output {
        let to_use_file = std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&(self.file))?;
        let mut serializer = serde_json::Serializer::new(BufWriter::new(to_use_file));
        self.history.serialize(&mut serializer)?;
        Ok(())
    }
}

impl RollBackable for HistoryWFile {
    type Output = ManagerCoreResult<()>;
    fn rollback(&mut self) -> Self::Output {
        let history_string_value = std::fs::read_to_string(&(self.file))?;
        self.history = serde_json::from_str::<HistoryBase>(&history_string_value)?;
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
            if error == HistoryBaseError::AlreadyExists(input) {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::HistoryBase(error))
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
            if error == HistoryBaseError::NotFound(input) {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::HistoryBase(error))
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
            if error == HistoryBaseError::AlreadyExists(input.id) {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::HistoryBase(error))
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
            if error == HistoryBaseError::NotFound(input.id) {
                if let Err(error) = self.commit() {
                    self.rollback()?;
                    Err(Error::RollBacked(error.to_string()))
                } else {
                    Ok(())
                }
            } else {
                Err(Error::HistoryBase(error))
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
        Ok(self.history.is_in(to_use).is_some())
    }
}

impl IsIn<HistoryEntry> for HistoryWFile {
    type Output = <HistoryBase as IsIn<HistoryEntry>>::Output;

    fn is_in(&self, to_use: HistoryEntry) -> Self::Output {
        self.history.is_in(to_use)
    }
}
