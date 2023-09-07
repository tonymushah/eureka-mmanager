use futures::Future;
use std::{ops::Deref, sync::Arc};
use tokio::sync::oneshot::channel;
use tokio::{
    sync::Mutex,
    task::{AbortHandle, JoinSet},
};

use crate::core::{Error, ManagerCoreResult};

pub mod chapter;
pub mod cover;
pub mod manga;

#[derive(Clone)]
pub struct DownloadTaks {
    tasks: Arc<Mutex<JoinSet<()>>>,
    limit: u16,
}

impl Default for DownloadTaks {
    fn default() -> Self {
        Self {
            tasks: Arc::new(Mutex::new(JoinSet::default())),
            limit: 20,
        }
    }
}

impl Deref for DownloadTaks {
    type Target = Mutex<JoinSet<()>>;

    fn deref(&self) -> &Self::Target {
        self.tasks.as_ref()
    }
}
impl DownloadTaks {
    pub fn new(limit: u16) -> Self {
        Self {
            tasks: Arc::new(Mutex::new(JoinSet::default())),
            limit,
        }
    }
    pub async fn verify_limit(&self) -> bool {
        self.lock().await.len() >= <u16 as Into<usize>>::into(self.limit)
    }
    pub async fn spawn<F>(&mut self, task: F) -> ManagerCoreResult<AbortHandle>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.verify_limit().await {
            Err(Error::DownloadTaskLimitExceded {
                current: self.lock().await.len().try_into()?,
                limit: self.limit,
            })
        } else {
            Ok(self.tasks.lock().await.spawn(task))
        }
    }
    pub async fn lock_spawn<F>(&mut self, task: F) -> AbortHandle
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut tasks = self.tasks.lock().await;
        if self.verify_limit().await {
            tasks.join_next().await;
            tasks.spawn(task)
        } else {
            tasks.spawn(task)
        }
    }
    pub async fn spawn_with_data<T>(&mut self, task: T) -> ManagerCoreResult<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let (sender, receiver) = channel::<T::Output>();
        self.spawn(async {
            let _ = sender.send(task.await);
        })
        .await?;
        Ok(receiver.await?)
    }
    pub async fn lock_spawn_with_data<T>(&mut self, task: T) -> ManagerCoreResult<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        let (sender, receiver) = channel::<T::Output>();
        self.lock_spawn(async {
            let _ = sender.send(task.await);
        })
        .await;
        Ok(receiver.await?)
    }
}
