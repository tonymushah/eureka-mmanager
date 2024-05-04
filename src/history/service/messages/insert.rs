use std::ops::{Deref, DerefMut};

use actix::prelude::*;

use crate::{
    history::{
        history_w_file::traits::{AsyncAutoCommitRollbackInsert, AutoCommitRollbackInsert},
        service::HistoryActorService,
        AsyncInsert, HistoryEntry, Insert,
    },
    ManagerCoreResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct InsertMessage {
    entry: HistoryEntry,
    auto_commit: bool,
}

impl InsertMessage {
    pub fn new(entry: HistoryEntry) -> Self {
        Self {
            entry,
            auto_commit: true,
        }
    }
    pub fn no_commit(self) -> Self {
        Self {
            entry: self.entry,
            auto_commit: false,
        }
    }
    pub fn commit(self) -> Self {
        Self {
            entry: self.entry,
            auto_commit: true,
        }
    }
}

impl Deref for InsertMessage {
    type Target = HistoryEntry;
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl DerefMut for InsertMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entry
    }
}

impl From<InsertMessage> for HistoryEntry {
    fn from(value: InsertMessage) -> Self {
        value.entry
    }
}

impl From<HistoryEntry> for InsertMessage {
    fn from(value: HistoryEntry) -> Self {
        InsertMessage::new(value)
    }
}

impl Message for InsertMessage {
    type Result = ManagerCoreResult<()>;
}

impl Handler<InsertMessage> for HistoryActorService {
    type Result = ManagerCoreResult<()>;
    fn handle(&mut self, msg: InsertMessage, _ctx: &mut Self::Context) -> Self::Result {
        if msg.auto_commit {
            <Self as AutoCommitRollbackInsert<HistoryEntry>>::insert(self, msg.entry)
        } else {
            <Self as Insert<HistoryEntry>>::insert(self, msg.entry)
        }
    }
}

impl<'a> AsyncAutoCommitRollbackInsert<'a, HistoryEntry> for Addr<HistoryActorService> {
    type Output = ManagerCoreResult<()>;
    async fn insert(
        &'a mut self,
        value: HistoryEntry,
    ) -> <Self as AsyncAutoCommitRollbackInsert<HistoryEntry>>::Output {
        self.send(InsertMessage::new(value)).await?
    }
}

impl<'a> AsyncInsert<'a, HistoryEntry> for Addr<HistoryActorService> {
    type Output = ManagerCoreResult<()>;
    async fn insert(
        &'a mut self,
        value: HistoryEntry,
    ) -> <Self as AsyncInsert<HistoryEntry>>::Output {
        self.send(InsertMessage::new(value).no_commit()).await?
    }
}
