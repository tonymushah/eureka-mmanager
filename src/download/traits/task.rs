use actix::prelude::*;
use tokio::sync::watch::Receiver;

use crate::download::state::{TaskState, WaitForFinished};

pub trait Cancelable: Actor {
    fn cancel(&mut self, ctx: &mut Self::Context);
}

pub trait Download: Actor {
    fn download(&mut self, ctx: &mut Self::Context);
}

pub trait State: Actor
where
    Self::State: Into<TaskState>,
{
    type State;
    fn state(&self) -> TaskState {
        self.inner_state().into()
    }
    fn inner_state(&self) -> Self::State;
}

pub trait Subscribe: State {
    fn subscribe(&mut self) -> crate::ManagerCoreResult<Receiver<Self::State>>;
}

pub trait CanBeWaited: State {
    type Ok;
    type Loading;
    fn wait(&mut self) -> WaitForFinished<Self::Ok, Self::Loading>;
}

pub trait AsyncCancelable {
    async fn cancel(&self);
}

pub trait AsyncDownload {
    async fn download(&self);
}

pub trait AsyncState
where
    Self::State: Into<TaskState>,
{
    type State;
    async fn state(&self) -> TaskState {
        self.inner_state().await.into()
    }
    async fn inner_state(&self) -> Self::State;
}

pub trait AsyncSubscribe: AsyncState {
    async fn subscribe(&mut self) -> crate::ManagerCoreResult<Receiver<Self::State>>;
}

pub trait AsyncCanBeWaited: AsyncState {
    type Ok;
    type Loading;
    async fn wait(&mut self) -> WaitForFinished<Self::Ok, Self::Loading>;
}
