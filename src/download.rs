use futures::Future;
use tokio::sync::{Semaphore, RwLock};
use std::fmt::Debug;
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
    sephamore : Arc<Semaphore>,
    on_lock : Arc<RwLock<usize>>
}

impl Default for DownloadTaks {
    fn default() -> Self {
        let limit = 20;
        Self {
            tasks: Arc::new(Mutex::new(JoinSet::default())),
            limit,
            sephamore : Arc::new(Semaphore::new(limit.into())),
            on_lock : Arc::new(RwLock::new(0))
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
            sephamore : Arc::new(Semaphore::new(limit.into())),
            on_lock : Arc::new(RwLock::new(0))
        }
    }
    pub async fn verify_limit(&self) -> bool {
        self.sephamore.available_permits() == 0
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
            let permit = self.sephamore.clone().acquire_owned().await?;
            Ok(self.tasks.lock().await.spawn(async move {
                task.await;
                drop(permit)
            }))
        }
    }
    async fn add_something_to_lock_list(&self) -> ManagerCoreResult<()> {
        let mut data = self.on_lock.write().await;
        *data += 1;
        Ok(())
    }
    async fn remove_something_to_lock_list(&self) -> ManagerCoreResult<()> {
        let mut data = self.on_lock.write().await;
        *data -= 1;
        Ok(())
    }
    pub async fn lock_spawn<F>(&mut self, task: F) -> ManagerCoreResult<AbortHandle>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        if self.verify_limit().await {
            let mut tasks = self.tasks.lock().await;
            self.add_something_to_lock_list().await?;
            tasks.join_next().await;
            self.remove_something_to_lock_list().await?;
            let permit = self.sephamore.clone().acquire_owned().await?;
            Ok(tasks.spawn(async move {
                task.await;
                drop(permit)
            }))
        } else {
            let mut tasks = self.tasks.lock().await;
            let permit = self.sephamore.clone().acquire_owned().await?;
            Ok(tasks.spawn(async move {
                task.await;
                drop(permit)
            }))
        }
    }
    pub async fn spawn_with_data<T>(&mut self, task: T) -> ManagerCoreResult<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + Debug + 'static,
    {
        let (sender, receiver) = channel::<T::Output>();
        self.spawn(async {
            match sender.send(task.await) {
                Ok(_) => {},
                Err(er) => println!("{:?}", er)
            };
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
        .await?;
        Ok(receiver.await?)
    }
    pub fn get_limit(&self) -> u16 {
        self.limit
    }
    pub fn get_running_tasks(&self) -> usize {
        <u16 as Into<usize>>::into(self.limit) - self.sephamore.available_permits()
    }
    pub async fn get_locked_tasks(&self) -> usize {
        *self.on_lock.read().await
    }
}
