use std::io::{ErrorKind, Write};

use mangadex_api_types_rust::RelationshipType;

use crate::{core::ManagerCoreResult, settings::files_dirs::DirsOptions, Error};

use self::traits::{AutoCommitRollbackInsert, AutoCommitRollbackRemove, Commitable, RollBackable};

use super::{History, HistoryEntry, Insert, Remove, IsIn};

pub mod traits;

#[derive(Clone, Debug)]
pub struct HistoryWFile {
    history: History,
    file: String,
}

impl HistoryWFile {
    pub fn new(data_type: RelationshipType, file: String) -> HistoryWFile {
        HistoryWFile {
            history: History::new(data_type),
            file,
        }
    }
    pub fn get_history(&mut self) -> &mut History {
        &mut (self.history)
    }
    pub fn get_file(self) -> String {
        self.file
    }
    pub fn from_file(file: String) -> Result<Self, std::io::Error> {
        let file_data: String = std::fs::read_to_string(file.clone())?;
        let history: History = serde_json::from_str(file_data.as_str())?;
        Ok(Self { history, file })
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
        <History as Insert<uuid::Uuid>>::insert(self.get_history(), input)
    }
}

impl Insert<HistoryEntry> for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn insert(&mut self, input: HistoryEntry) -> Self::Output {
        <History as Insert<HistoryEntry>>::insert(self.get_history(), input)
    }
}

impl Remove<uuid::Uuid> for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn remove(&mut self, input: uuid::Uuid) -> Self::Output {
        <History as Remove<uuid::Uuid>>::remove(self.get_history(), input)
    }
}

impl Remove<HistoryEntry> for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn remove(&mut self, input: HistoryEntry) -> Self::Output {
        <History as Remove<HistoryEntry>>::remove(self.get_history(), input)
    }
}

impl Commitable for HistoryWFile {
    type Output = Result<(), std::io::Error>;

    fn commit(&mut self) -> Self::Output {
        let history_string_value = serde_json::to_string(&(self.history))?;
        let mut to_use_file = std::fs::File::options()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&(self.file))?;
        to_use_file.write_all(history_string_value.as_bytes())?;
        Ok(())
    }
}

impl RollBackable for HistoryWFile {
    type Output = Result<(), std::io::Error>;
    fn rollback(&mut self) -> Self::Output {
        let history_string_value = std::fs::read_to_string(&(self.file))?;
        self.history = serde_json::from_str::<History>(&history_string_value)?;
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

    fn insert(&mut self, input : HistoryEntry) -> <Self as crate::settings::file_history::history_w_file::traits::AutoCommitRollbackInsert<HistoryEntry>>::Output {
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

impl IsIn<uuid::Uuid> for HistoryWFile{
    type Output = bool;
    
    fn is_in(&self, to_use : uuid::Uuid) -> Self::Output {
        <History as IsIn<uuid::Uuid>>::is_in(&self.history, to_use)
    }
}

impl IsIn<HistoryEntry> for HistoryWFile{
    type Output = <History as IsIn<HistoryEntry>>::Output;

    fn is_in(&self, to_use : HistoryEntry) -> Self::Output {
        <History as IsIn<HistoryEntry>>::is_in(&self.history, to_use)
    }
}