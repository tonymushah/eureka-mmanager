use std::{future::Future, ops::Deref, task::Poll};

use actix::prelude::*;
use tokio::sync::watch::{self, Receiver, Sender};
use tokio_util::sync::ReusableBoxFuture;

use crate::{download::messages::TaskSubscriberMessages, ManagerCoreResult, OwnedError};

#[derive(Debug, Clone)]
pub enum DownloadTaskState<T, L> {
    Pending,
    Loading(L),
    Error(OwnedError),
    Done(T),
    Canceled,
}

impl<T, L> Default for DownloadTaskState<T, L> {
    fn default() -> Self {
        Self::Pending
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, MessageResponse)]
pub enum TaskState {
    Pending,
    Loading,
    Error,
    Done,
    Canceled,
}

impl Default for TaskState {
    fn default() -> Self {
        Self::Pending
    }
}

impl TaskState {
    pub fn is_finished(&self) -> bool {
        *self == Self::Done || *self == Self::Canceled || *self == Self::Error
    }
    pub fn is_pending(&self) -> bool {
        matches!(*self, Self::Pending)
    }
    pub fn is_loading(&self) -> bool {
        matches!(*self, Self::Loading)
    }
}

impl<T, L> From<DownloadTaskState<T, L>> for TaskState {
    fn from(value: DownloadTaskState<T, L>) -> Self {
        (&value).into()
    }
}

impl<T, L> From<&DownloadTaskState<T, L>> for TaskState {
    fn from(value: &DownloadTaskState<T, L>) -> Self {
        match value {
            DownloadTaskState::Pending => Self::Pending,
            DownloadTaskState::Loading(_) => Self::Loading,
            DownloadTaskState::Error(_) => Self::Error,
            DownloadTaskState::Done(_) => Self::Done,
            DownloadTaskState::Canceled => Self::Canceled,
        }
    }
}

impl<T, L> From<ManagerCoreResult<T>> for DownloadTaskState<T, L> {
    fn from(value: ManagerCoreResult<T>) -> Self {
        match value {
            Ok(v) => Self::Done(v),
            Err(v) => Self::Error(v.into()),
        }
    }
}

struct WaitForFinishedActor<T, L> {
    state: Sender<DownloadTaskState<T, L>>,
}

impl<T, L> Actor for WaitForFinishedActor<T, L>
where
    T: 'static,
    L: 'static,
{
    type Context = Context<Self>;
}

impl<T, L> Handler<TaskSubscriberMessages<DownloadTaskState<T, L>>> for WaitForFinishedActor<T, L>
where
    T: 'static,
    L: 'static,
{
    type Result = ();
    fn handle(
        &mut self,
        msg: TaskSubscriberMessages<DownloadTaskState<T, L>>,
        ctx: &mut Self::Context,
    ) -> Self::Result {
        if self.state.is_closed() {
            ctx.stop();
            return;
        }
        match msg {
            TaskSubscriberMessages::State(s) => {
                let _ = self.state.send_replace(s);
            }
            TaskSubscriberMessages::ID(_) => {
                let _ = self.state.send_replace(DownloadTaskState::Pending);
            }
            TaskSubscriberMessages::Dropped => {
                let state = Into::<TaskState>::into(self.state.borrow().deref());
                if !state.is_finished() {
                    let _ = self.state.send_replace(DownloadTaskState::Canceled);
                }
            }
        };
    }
}

type WaitForFinishedCouple<T, L> = (
    Recipient<TaskSubscriberMessages<DownloadTaskState<T, L>>>,
    WaitForFinished<T, L>,
);

pub(crate) fn make_wait_for_finish_couple<T, L>() -> WaitForFinishedCouple<T, L>
where
    T: 'static + Send + Clone + Sync,
    L: 'static + Send + Sync,
{
    let (tx, rx) = watch::channel(DownloadTaskState::Pending);
    (
        WaitForFinishedActor { state: tx }.start().recipient(),
        WaitForFinished::new(rx),
    )
}

#[derive(Debug, MessageResponse)]
pub struct WaitForFinished<T, L> {
    state: Receiver<DownloadTaskState<T, L>>,

    fut: ReusableBoxFuture<'static, Result<T, WaitForFinishedError>>,
}

async fn make_future<T: Clone + Send + Sync, L: Send + Sync>(
    mut rx: Receiver<DownloadTaskState<T, L>>,
) -> Result<T, WaitForFinishedError> {
    loop {
        rx.changed()
            .await
            .map_err(WaitForFinishedError::RecvError)?;
        match rx.borrow().deref() {
            DownloadTaskState::Error(e) => {
                return Err(WaitForFinishedError::Error(e.clone()));
            }
            DownloadTaskState::Done(d) => return Ok(d.clone()),
            DownloadTaskState::Canceled => return Err(WaitForFinishedError::Canceled),
            _ => {}
        }
    }
}

impl<T, L> WaitForFinished<T, L>
where
    T: Clone + Send + Sync + 'static,
    L: Send + Sync + 'static,
{
    pub fn new(state: Receiver<DownloadTaskState<T, L>>) -> Self {
        let mut rx = state.clone();
        rx.mark_changed();
        Self {
            state,
            fut: ReusableBoxFuture::new(make_future(rx)),
        }
    }
}

impl<T, L> Clone for WaitForFinished<T, L>
where
    T: Clone + Send + Sync + 'static,
    L: Send + Sync + 'static,
{
    fn clone(&self) -> Self {
        Self::new(self.state.clone())
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WaitForFinishedError {
    #[error("The task was been canceled")]
    Canceled,
    #[error("{0}")]
    Error(OwnedError),
    #[error(transparent)]
    RecvError(#[from] tokio::sync::watch::error::RecvError),
}

impl<T, L> Future for WaitForFinished<T, L>
where
    T: Send + Sync,
{
    type Output = Result<T, WaitForFinishedError>;
    fn poll(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> Poll<Self::Output> {
        self.fut.poll(cx)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum DownloadMessageState {
    Pending,
    Downloading,
}

impl Default for DownloadMessageState {
    fn default() -> Self {
        Self::Pending
    }
}
