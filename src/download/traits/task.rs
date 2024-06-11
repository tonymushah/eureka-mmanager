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
    fn cancel(&self) -> impl std::future::Future<Output = ()> + Send;
}

pub trait AsyncDownload {
    fn download(&self) -> impl std::future::Future<Output = ()> + Send;
}

pub trait AsyncState
where
    Self: Sync,
    Self::State: Into<TaskState>,
{
    type State;
    fn state(&self) -> impl std::future::Future<Output = TaskState> + Send {
        async { self.inner_state().await.into() }
    }
    fn inner_state(&self) -> impl std::future::Future<Output = Self::State> + Send;
}

pub trait AsyncSubscribe: AsyncState {
    fn subscribe(
        &mut self,
    ) -> impl std::future::Future<Output = crate::ManagerCoreResult<Receiver<Self::State>>> + Send;
}

pub trait AsyncCanBeWaited: AsyncState {
    type Ok;
    type Loading;
    fn wait(
        &mut self,
    ) -> impl std::future::Future<Output = WaitForFinished<Self::Ok, Self::Loading>> + Send;
}
