use std::ops::{Deref, DerefMut};

use actix::prelude::*;

use crate::{
    history::{service::HistoryActorService, AsyncIsIn, HistoryEntry, IsIn},
    MailBoxResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IsInMessage(pub HistoryEntry);

impl Deref for IsInMessage {
    type Target = HistoryEntry;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for IsInMessage {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl From<IsInMessage> for HistoryEntry {
    fn from(value: IsInMessage) -> Self {
        value.0
    }
}

impl From<HistoryEntry> for IsInMessage {
    fn from(value: HistoryEntry) -> Self {
        IsInMessage(value)
    }
}

impl Message for IsInMessage {
    type Result = bool;
}

impl Handler<IsInMessage> for HistoryActorService {
    type Result = bool;
    fn handle(&mut self, msg: IsInMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.is_in(msg.0)
    }
}

impl<'a, T> AsyncIsIn<'a, T> for Addr<HistoryActorService>
where
    T: Into<HistoryEntry>,
{
    type Output = MailBoxResult<bool>;
    fn is_in(&'a self, to_use: T) -> impl std::future::Future<Output = Self::Output> + Send {
        self.send(Into::<IsInMessage>::into(to_use.into()))
    }
}
