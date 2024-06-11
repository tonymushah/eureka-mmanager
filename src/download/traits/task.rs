use actix::prelude::*;
use dev::ToEnvelope;
use tokio::sync::watch::Receiver;

use crate::download::{
    messages::{CancelTaskMessage, StartDownload, TaskStateMessage},
    state::{TaskState, WaitForFinished},
};

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

type MailBoxResult<T, E = MailboxError> = Result<T, E>;

pub trait AsyncCancelable {
    fn cancel(&self) -> impl std::future::Future<Output = MailBoxResult<()>> + Send;
}

impl<A> AsyncCancelable for Addr<A>
where
    A: Handler<CancelTaskMessage> + Cancelable,
    <A as actix::Actor>::Context:
        actix::dev::ToEnvelope<A, crate::download::messages::CancelTaskMessage>,
{
    async fn cancel(&self) -> MailBoxResult<()> {
        self.send(CancelTaskMessage).await
    }
}

pub trait AsyncDownload {
    fn download(&self) -> impl std::future::Future<Output = MailBoxResult<()>> + Send;
}

impl<A> AsyncDownload for Addr<A>
where
    A: Handler<StartDownload> + Download,
    <A as Actor>::Context: ToEnvelope<A, StartDownload>,
{
    async fn download(&self) -> MailBoxResult<()> {
        self.send(StartDownload).await
    }
}

pub trait AsyncState {
    type State;
    fn state(&self) -> impl std::future::Future<Output = MailBoxResult<TaskState>> + Send;
}

impl<A> AsyncState for Addr<A>
where
    A: Handler<TaskStateMessage> + State,
    <A as Actor>::Context: ToEnvelope<A, TaskStateMessage>,
{
    type State = <A as State>::State;
    async fn state(&self) -> MailBoxResult<TaskState> {
        self.send(TaskStateMessage).await
    }
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
