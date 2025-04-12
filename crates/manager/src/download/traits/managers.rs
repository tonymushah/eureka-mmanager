use std::{future::Future, sync::Arc};

use actix::prelude::*;
use dev::ToEnvelope;
use tokio::sync::Notify;
use uuid::Uuid;

use super::MailBoxResult;

use crate::download::{
    messages::{
        state::GetManagerStateMessage, DropSingleTaskMessage, GetTaskMessage, GetTasksListMessage,
        SubcribeToManagerMessage,
    },
    state::DownloadManagerState,
};

pub trait TaskManager: Actor
where
    Self::Task: Actor,
    Self::DownloadMessage: Message<Result = Addr<Self::Task>>,
{
    type Task;
    type DownloadMessage;
    fn state(&self) -> Addr<DownloadManagerState>;
    fn notify(&self) -> Arc<Notify>;
    fn tasks_id(&self) -> Vec<Uuid>;
    fn tasks(&self) -> Vec<Addr<Self::Task>>;
    fn new_task(&mut self, msg: Self::DownloadMessage, ctx: &mut Self::Context)
        -> Addr<Self::Task>;
    fn drop_task(&mut self, id: Uuid);
    fn get_task(&self, id: Uuid) -> Option<Addr<Self::Task>>;
}

pub trait TaskManagerAddr: Sync
where
    Self::Task: Actor,
{
    type Task;
    type DownloadMessage;
    fn state(&self) -> impl Future<Output = MailBoxResult<Addr<DownloadManagerState>>> + Send;
    fn notify(&self) -> impl Future<Output = MailBoxResult<Arc<Notify>>> + Send;
    fn tasks_id(&self) -> impl Future<Output = MailBoxResult<Vec<Uuid>>> + Send;
    // TODO Add this into a future api change
    //fn tasks(&self) -> impl Future<Output = MailBoxResult<Vec<Addr<Self::Task>>>> + Send;
    fn new_task(
        &self,
        msg: Self::DownloadMessage,
    ) -> impl Future<Output = MailBoxResult<Addr<Self::Task>>> + Send;
    fn drop_task(&self, id: Uuid) -> impl Future<Output = MailBoxResult<()>> + Send;
    fn get_task(
        &self,
        id: Uuid,
    ) -> impl Future<Output = MailBoxResult<Option<Addr<Self::Task>>>> + Send;
}

impl<T> TaskManagerAddr for Addr<T>
where
    T: TaskManager,
    T: Handler<GetManagerStateMessage>
        + Handler<SubcribeToManagerMessage>
        + Handler<GetTasksListMessage>
        + Handler<DropSingleTaskMessage>
        + Handler<T::DownloadMessage>
        + Handler<GetTaskMessage<T::Task>>,
    T::DownloadMessage: Send,
    <T as Actor>::Context: ToEnvelope<T, GetManagerStateMessage>
        + ToEnvelope<T, SubcribeToManagerMessage>
        + ToEnvelope<T, GetTasksListMessage>
        + ToEnvelope<T, DropSingleTaskMessage>
        + ToEnvelope<T, T::DownloadMessage>
        + ToEnvelope<T, GetTaskMessage<T::Task>>,
{
    type DownloadMessage = T::DownloadMessage;
    type Task = T::Task;

    async fn state(&self) -> MailBoxResult<Addr<DownloadManagerState>> {
        self.send(GetManagerStateMessage).await
    }

    async fn notify(&self) -> MailBoxResult<Arc<Notify>> {
        self.send(SubcribeToManagerMessage).await
    }

    async fn tasks_id(&self) -> MailBoxResult<Vec<Uuid>> {
        self.send(GetTasksListMessage).await
    }

    async fn new_task(&self, msg: Self::DownloadMessage) -> MailBoxResult<Addr<Self::Task>> {
        self.send(msg).await
    }
    async fn drop_task(&self, id: Uuid) -> MailBoxResult<()> {
        self.send(DropSingleTaskMessage(id)).await
    }
    async fn get_task(&self, id: Uuid) -> MailBoxResult<Option<Addr<Self::Task>>> {
        self.send(GetTaskMessage::<Self::Task>::new(id)).await
    }
}
