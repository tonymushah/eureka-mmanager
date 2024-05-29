use std::{collections::HashMap, fs::read_dir};

use actix::prelude::*;
use mangadex_api_types_rust::RelationshipType;

use crate::{DirsOptions, JoinHistoryMessage, ManagerCoreResult};

use super::{HistoryEntry, HistoryWFile, Remove};

pub mod messages;

use crate::history::{
    history_w_file::traits::{AutoCommitRollbackRemove, Commitable, RollBackable},
    IsIn,
};

#[derive(Debug, Clone)]
pub struct HistoryActorService {
    dirs: Addr<DirsOptions>,
    files: HashMap<RelationshipType, HistoryWFile>,
}

impl HistoryActorService {
    pub fn new_blocking(dirs: Addr<DirsOptions>) -> Self {
        Self {
            dirs,
            files: Default::default(),
        }
    }
    pub async fn load_files(&mut self) {
        self.files = self
            .dirs
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
    }
    pub async fn new(dirs: Addr<DirsOptions>) -> Self {
        let mut this = Self::new_blocking(dirs);
        this.load_files().await;
        this
    }
    fn get_history(&self, rel: RelationshipType) -> Option<&HistoryWFile> {
        self.files.get(&rel)
    }
    fn get_history_mut(&mut self, rel: RelationshipType) -> Option<&mut HistoryWFile> {
        self.files.get_mut(&rel)
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
