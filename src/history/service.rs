use std::{collections::HashMap, fs::read_dir};

use actix::prelude::*;
use mangadex_api_types_rust::RelationshipType;

use crate::{DirsOptions, JoinHistoryMessage, ManagerCoreResult};

use super::{HistoryEntry, HistoryWFile, Remove};

pub mod messages;

use crate::history::{
    history_w_file::traits::{
        AutoCommitRollbackInsert, AutoCommitRollbackRemove, Commitable, RollBackable,
    },
    Insert, IsIn,
};

#[derive(Debug, Clone)]
pub struct HistoryActorService {
    dirs: Addr<DirsOptions>,
    files: HashMap<RelationshipType, HistoryWFile>,
}

impl HistoryActorService {
    pub async fn new(dirs: Addr<DirsOptions>) -> Self {
        let files = dirs
            .send(Into::<JoinHistoryMessage<&'static str>>::into(""))
            .await
            .map(|p| {
                read_dir(p)
                    .map(|dir| {
                        dir.flatten()
                            .flat_map(|f| HistoryWFile::from_file(f.path()))
                            .collect::<Vec<HistoryWFile>>()
                    })
                    .unwrap_or_default()
            })
            .unwrap_or_default()
            .into_iter()
            .map(|file| (*file.get_history().get_data_type(), file))
            .collect();

        Self { files, dirs }
    }
    fn get_history(&self, rel: RelationshipType) -> Option<&HistoryWFile> {
        self.files.get(&rel)
    }
    fn get_history_mut(&mut self, rel: RelationshipType) -> Option<&mut HistoryWFile> {
        self.files.get_mut(&rel)
    }
    fn get_history_or_init(
        &mut self,
        rel: RelationshipType,
    ) -> ManagerCoreResult<&mut HistoryWFile> {
        if let std::collections::hash_map::Entry::Occupied(mut e) = self.files.entry(rel) {
            e.insert(HistoryWFile::init(rel, self.dirs.clone())?);
        }
        self.get_history_mut(rel)
            .ok_or(crate::Error::HistoryFileNotFound(rel))
    }
}

impl IsIn<HistoryEntry> for HistoryActorService {
    type Output = bool;
    fn is_in(&self, to_use: HistoryEntry) -> Self::Output {
        self.get_history(to_use.data_type)
            .map(|h| h.is_in(to_use).unwrap_or(false))
            .unwrap_or(false)
    }
}

impl Insert<HistoryEntry> for HistoryActorService {
    type Output = ManagerCoreResult<()>;
    fn insert(&mut self, input: HistoryEntry) -> Self::Output {
        let file = self.get_history_or_init(input.data_type)?;
        <HistoryWFile as Insert<HistoryEntry>>::insert(file, input)?;
        Ok(())
    }
}

impl Remove<HistoryEntry> for HistoryActorService {
    type Output = ManagerCoreResult<()>;
    fn remove(&mut self, input: HistoryEntry) -> Self::Output {
        let file = self
            .get_history_mut(input.data_type)
            .ok_or(crate::Error::HistoryFileNotFound(input.data_type))?;
        <HistoryWFile as Remove<HistoryEntry>>::remove(file, input)?;
        Ok(())
    }
}

impl Commitable for HistoryActorService {
    type Output = ManagerCoreResult<()>;
    fn commit(&self) -> Self::Output {
        for (_, file) in self.files.iter() {
            <HistoryWFile as Commitable>::commit(file)?;
        }
        Ok(())
    }
}

impl RollBackable for HistoryActorService {
    type Output = ManagerCoreResult<()>;
    fn rollback(&mut self) -> Self::Output {
        for (_, file) in self.files.iter_mut() {
            <HistoryWFile as RollBackable>::rollback(file)?;
        }
        Ok(())
    }
}

impl AutoCommitRollbackInsert<HistoryEntry> for HistoryActorService {
    type Output = ManagerCoreResult<()>;
    fn insert(
        &mut self,
        input: HistoryEntry,
    ) -> <Self as AutoCommitRollbackInsert<HistoryEntry>>::Output {
        let file = self.get_history_or_init(input.data_type)?;
        <HistoryWFile as AutoCommitRollbackInsert<HistoryEntry>>::insert(file, input)?;
        Ok(())
    }
}

impl AutoCommitRollbackRemove<HistoryEntry> for HistoryActorService {
    type Output = ManagerCoreResult<()>;
    fn remove(
        &mut self,
        input: HistoryEntry,
    ) -> <Self as AutoCommitRollbackRemove<HistoryEntry>>::Output {
        let file = self
            .get_history_mut(input.data_type)
            .ok_or(crate::Error::HistoryFileNotFound(input.data_type))?;
        <HistoryWFile as AutoCommitRollbackRemove<HistoryEntry>>::remove(file, input)?;
        Ok(())
    }
}

impl Actor for HistoryActorService {
    type Context = Context<Self>;
    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let _ = self.commit();
    }
}
