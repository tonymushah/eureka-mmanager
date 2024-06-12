use std::sync::Arc;

use actix::prelude::*;
use tokio::sync::Notify;
use uuid::Uuid;

use crate::download::state::DownloadManagerState;

pub trait DownloadManager: Actor
where
    Self::Task: Actor,
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
