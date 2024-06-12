use std::sync::Arc;

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use crate::download::state::DownloadManagerState;

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
}
