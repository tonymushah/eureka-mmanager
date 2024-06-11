use actix::prelude::*;
use tokio::sync::watch::Receiver;

use super::state::{TaskState, WaitForFinished};

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
