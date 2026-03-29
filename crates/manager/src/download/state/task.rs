#[cfg(test)]
mod test_wait;

use std::{future::Future, marker::PhantomData, task::Poll};

use actix::prelude::*;
use tokio::sync::mpsc::{self, UnboundedReceiver, UnboundedSender};
use tokio_util::sync::ReusableBoxFuture;

use crate::{ManagerCoreResult, OwnedError, download::messages::TaskSubscriberMessages};

#[derive(Debug, Clone, Default)]
pub enum DownloadTaskState<T, L> {
    #[default]
    Pending,
    Loading(L),
    Error(OwnedError),
    Done(T),
    Canceled,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, MessageResponse, Default)]
pub enum TaskState {
    #[default]
    Pending,
    Loading,
    Error,
    Done,
    Canceled,
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
    tx: UnboundedSender<DownloadTaskState<T, L>>,
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
        if self.tx.is_closed() {
            ctx.stop();
            log::warn!("Stopping actor since the channel closed");
            return;
        }
        let res = match msg {
            TaskSubscriberMessages::State(s) => self.tx.send(s),
            TaskSubscriberMessages::ID(_) => self.tx.send(DownloadTaskState::Pending),
            TaskSubscriberMessages::Dropped => self.tx.send(DownloadTaskState::Canceled),
        };
        if res.is_err() {
            log::warn!("Stopping actor since the channel closed");
        }
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
    let (tx, rx) = mpsc::unbounded_channel();
    let _ = tx.send(DownloadTaskState::Pending);
    (
        WaitForFinishedActor { tx }.start().recipient(),
        WaitForFinished::new(rx),
    )
}

#[derive(Debug, MessageResponse)]
pub struct WaitForFinished<T, L> {
    phantom: PhantomData<L>,
    fut: ReusableBoxFuture<'static, Result<T, WaitForFinishedError>>,
}

async fn make_future<T: Clone + Send + Sync, L: Send + Sync>(
    mut rx: UnboundedReceiver<DownloadTaskState<T, L>>,
) -> Result<T, WaitForFinishedError> {
    loop {
        let val = rx.recv().await.ok_or(WaitForFinishedError::ChannelClosed)?;
        match val {
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
    pub fn new(state: UnboundedReceiver<DownloadTaskState<T, L>>) -> Self {
        Self {
            phantom: PhantomData,
            fut: ReusableBoxFuture::new(make_future(state)),
        }
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WaitForFinishedError {
    #[error("The task was been canceled")]
    Canceled,
    #[error("{0}")]
    Error(OwnedError),
    #[error("The mpsc is closed")]
    ChannelClosed,
}

impl<T, L> Future for WaitForFinished<T, L>
where
    T: Send + Sync,
    L: Unpin,
{
    type Output = Result<T, WaitForFinishedError>;
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        self.get_mut().fut.poll(cx)
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum DownloadMessageState {
    #[default]
    Pending,
    Downloading,
}
