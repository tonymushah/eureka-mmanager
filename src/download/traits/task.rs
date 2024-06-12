use actix::prelude::*;
use dev::ToEnvelope;
use tokio::sync::watch::Receiver;

use super::MailBoxResult;

use crate::download::{
    messages::{
        CancelTaskMessage, StartDownload, SubcribeMessage, TaskStateMessage, WaitForFinishedMessage,
    },
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

pub trait AsyncCancelable: Sync {
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

pub trait AsyncDownload: Sync {
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

pub trait AsyncState: Sync {
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
        &self,
    ) -> impl std::future::Future<Output = crate::ManagerCoreResult<Receiver<Self::State>>> + Send;
}

impl<A> AsyncSubscribe for Addr<A>
where
    A: Handler<SubcribeMessage<<A as State>::State>>
        + Subscribe
        + Handler<TaskStateMessage>
        + State,
    <A as State>::State: Send + Sync,
    <A as Actor>::Context:
        ToEnvelope<A, SubcribeMessage<<A as State>::State>> + ToEnvelope<A, TaskStateMessage>,
{
    async fn subscribe(&self) -> crate::ManagerCoreResult<Receiver<Self::State>> {
        self.send(SubcribeMessage::<Self::State>::new()).await?
    }
}

pub trait AsyncCanBeWaited: AsyncState {
    type Ok;
    type Loading;
    fn wait(
        &mut self,
    ) -> impl std::future::Future<Output = MailBoxResult<WaitForFinished<Self::Ok, Self::Loading>>> + Send;
}

impl<A> AsyncCanBeWaited for Addr<A>
where
    A: Handler<SubcribeMessage<<A as State>::State>>
        + Subscribe
        + Handler<TaskStateMessage>
        + State
        + CanBeWaited
        + Handler<WaitForFinishedMessage<<A as CanBeWaited>::Ok, <A as CanBeWaited>::Loading>>,
    <A as State>::State: Send + Sync,
    <A as CanBeWaited>::Loading: Send + Sync,
    <A as CanBeWaited>::Ok: Send + Sync,
    <A as Actor>::Context: ToEnvelope<A, SubcribeMessage<<A as State>::State>>
        + ToEnvelope<A, TaskStateMessage>
        + ToEnvelope<A, WaitForFinishedMessage<<A as CanBeWaited>::Ok, <A as CanBeWaited>::Loading>>,
{
    type Loading = <A as CanBeWaited>::Loading;
    type Ok = <A as CanBeWaited>::Ok;
    async fn wait(&mut self) -> MailBoxResult<WaitForFinished<Self::Ok, Self::Loading>> {
        self.send(WaitForFinishedMessage::<
            <A as CanBeWaited>::Ok,
            <A as CanBeWaited>::Loading,
        >::new())
            .await
    }
}
