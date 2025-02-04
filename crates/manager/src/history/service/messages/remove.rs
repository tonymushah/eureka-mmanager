use std::ops::{Deref, DerefMut};

use actix::prelude::*;

use crate::{
    history::{
        history_w_file::traits::{AsyncAutoCommitRollbackRemove, AutoCommitRollbackRemove},
        service::HistoryActorService,
        AsyncRemove, HistoryEntry, Remove,
    },
    ManagerCoreResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RemoveMessage {
    entry: HistoryEntry,
    auto_commit: bool,
}

impl RemoveMessage {
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

impl Deref for RemoveMessage {
    type Target = HistoryEntry;
    fn deref(&self) -> &Self::Target {
        &self.entry
    }
}

impl DerefMut for RemoveMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.entry
    }
}

impl From<RemoveMessage> for HistoryEntry {
    fn from(value: RemoveMessage) -> Self {
        value.entry
    }
}

impl From<HistoryEntry> for RemoveMessage {
    fn from(value: HistoryEntry) -> Self {
        RemoveMessage::new(value)
    }
}

impl Message for RemoveMessage {
    type Result = ManagerCoreResult<()>;
}

impl Handler<RemoveMessage> for HistoryActorService {
    type Result = ManagerCoreResult<()>;
    fn handle(&mut self, msg: RemoveMessage, _ctx: &mut Self::Context) -> Self::Result {
        if msg.auto_commit {
            <Self as AutoCommitRollbackRemove<HistoryEntry>>::remove(self, msg.entry)
        } else {
            <Self as Remove<HistoryEntry>>::remove(self, msg.entry)
        }
    }
}

impl<'a> AsyncAutoCommitRollbackRemove<'a, HistoryEntry> for Addr<HistoryActorService> {
    type Output = ManagerCoreResult<()>;
    async fn remove_and_commit(
        &'a mut self,
        value: HistoryEntry,
    ) -> <Self as AsyncAutoCommitRollbackRemove<'a, HistoryEntry>>::Output {
        self.send(RemoveMessage::new(value)).await?
    }
}

impl<'a> AsyncRemove<'a, HistoryEntry> for Addr<HistoryActorService> {
    type Output = ManagerCoreResult<()>;
    async fn remove(
        &'a mut self,
        value: HistoryEntry,
    ) -> <Self as AsyncRemove<'a, HistoryEntry>>::Output {
        self.send(RemoveMessage::new(value).no_commit()).await?
    }
}
