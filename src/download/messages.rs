use std::marker::PhantomData;

use actix::prelude::*;
use uuid::Uuid;

use super::state::{TaskState, WaitForFinished};

#[derive(Debug, Clone, Copy)]
pub struct DropSingleTaskMessage(pub Uuid);

impl Message for DropSingleTaskMessage {
    type Result = crate::ManagerCoreResult<()>;
}

#[derive(Debug, Default)]
pub struct SubcribeMessage<T: ?Sized>(PhantomData<T>);

impl<T: 'static> Message for SubcribeMessage<T> {
    type Result = crate::ManagerCoreResult<tokio::sync::watch::Receiver<T>>;
}

#[derive(Debug, Default, Clone, Copy, Message)]
#[rtype(result = "()")]
pub struct StartDownload;

#[derive(Debug, Default)]
pub struct WaitForFinishedMessage<T: ?Sized, L: ?Sized>(PhantomData<T>, PhantomData<L>);

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
