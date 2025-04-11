pub mod chapter;
pub mod cover;
pub mod manga;
pub mod state;

use std::{marker::PhantomData, sync::Arc};

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use super::state::{TaskState, WaitForFinished};

#[derive(Debug, Clone, Copy)]
pub struct DropSingleTaskMessage(pub Uuid);

impl Message for DropSingleTaskMessage {
    type Result = ();
}

#[derive(Debug, Default)]
pub struct SubcribeMessage<T: ?Sized>(PhantomData<T>);

impl<T: 'static> Message for SubcribeMessage<T> {
    type Result = crate::ManagerCoreResult<tokio::sync::watch::Receiver<T>>;
}

impl<T: ?Sized> SubcribeMessage<T> {
    pub fn new() -> Self {
        Self(PhantomData::<T>)
    }
}

#[derive(Debug, Default, Clone, Copy, Message)]
#[rtype(result = "()")]
pub struct StartDownload;

#[derive(Debug, Default)]
pub struct WaitForFinishedMessage<T: ?Sized, L: ?Sized>(PhantomData<T>, PhantomData<L>);

impl<T: ?Sized, L: ?Sized> WaitForFinishedMessage<T, L> {
    pub fn new() -> Self {
        Self(PhantomData::<T>, PhantomData::<L>)
    }
}

impl<T, L> Message for WaitForFinishedMessage<T, L>
where
    T: 'static,
    L: 'static,
{
    type Result = WaitForFinished<T, L>;
}

#[derive(Debug, Clone, Copy, Default, Message)]
#[rtype(result = "()")]
pub struct CancelTaskMessage;

#[derive(Debug, Clone, Copy, Default)]
pub struct TaskStateMessage;

impl Message for TaskStateMessage {
    type Result = TaskState;
}

pub struct GetTasksListMessage;

impl Message for GetTasksListMessage {
    type Result = Vec<Uuid>;
}

pub struct SubcribeToManagerMessage;

impl Message for SubcribeToManagerMessage {
    type Result = Arc<Notify>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GetTaskMessage<T> {
    id: Uuid,
    _phantom: PhantomData<T>,
}

impl<T> GetTaskMessage<T> {
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            _phantom: PhantomData,
        }
    }
}

impl<T> From<Uuid> for GetTaskMessage<T> {
    fn from(value: Uuid) -> Self {
        Self::new(value)
    }
}

impl<T> From<GetTaskMessage<T>> for Uuid {
    fn from(value: GetTaskMessage<T>) -> Self {
        value.id
    }
}

impl<T> Message for GetTaskMessage<T>
where
    T: Actor,
{
    type Result = Option<Addr<T>>;
}
