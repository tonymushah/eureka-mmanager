use std::{
    future::Future,
    ops::Deref,
    task::{ready, Poll},
};

use actix::prelude::*;
use tokio::sync::watch::Receiver;

use crate::{ManagerCoreResult, OwnedError};

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

#[derive(Debug, Clone, MessageResponse)]
pub struct WaitForFinished<T, L> {
    state: Receiver<DownloadTaskState<T, L>>,
    waker_on_load: bool,
}

impl<T, L> WaitForFinished<T, L> {
    pub fn new(state: Receiver<DownloadTaskState<T, L>>) -> Self {
        Self {
            state,
            waker_on_load: true,
        }
    }
    pub fn waker_on_load(self, waker_on_load: bool) -> Self {
        Self {
            waker_on_load,
            ..self
        }
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
    T: Clone,
{
    type Output = Result<T, WaitForFinishedError>;
    // TODO test WaitForFinished with and without cx.waker().wake_by_ref()
    fn poll(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> Poll<Self::Output> {
        let mut state = self.state.clone();
        let mut changed = Box::pin(state.changed());
        match ready!(changed.as_mut().poll(cx)) {
            Ok(_) => match self.state.borrow().deref() {
                DownloadTaskState::Pending => {
                    if self.waker_on_load {
                        cx.waker().wake_by_ref();
                    }
                    Poll::Pending
                }
                DownloadTaskState::Loading(_) => {
                    if self.waker_on_load {
                        cx.waker().wake_by_ref();
                    }
                    Poll::Pending
                }
                DownloadTaskState::Error(e) => {
                    Poll::Ready(Err(WaitForFinishedError::Error(e.clone())))
                }
                DownloadTaskState::Done(d) => Poll::Ready(Ok(d.clone())),
                DownloadTaskState::Canceled => Poll::Ready(Err(WaitForFinishedError::Canceled)),
            },
            Err(e) => Poll::Ready(Err(WaitForFinishedError::RecvError(e))),
        }
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
