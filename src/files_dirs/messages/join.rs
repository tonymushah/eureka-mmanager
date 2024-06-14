pub mod join_chapters;
pub mod join_covers;
pub mod join_covers_images;
pub mod join_data;
pub mod join_history;

use std::{
    fmt::Debug,
    future::Future,
    path::{Path, PathBuf},
};

use actix::Addr;

use crate::{download::state::messages::get::GetManagerStateData, DirsOptions, MailBoxResult};

pub use self::{
    join_chapters::JoinChaptersMessage, join_covers::JoinCoversMessage,
    join_covers_images::JoinCoversImagesMessage, join_data::JoinDataMessage,
    join_history::JoinHistoryMessage,
};

pub trait JoinPathAsyncTraits: Sync {
    fn join_chapters(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>>;
    fn join_covers(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>>;
    fn join_covers_images(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>>;
    fn join_data(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>>;
    fn join_history(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>>;
}

impl JoinPathAsyncTraits for Addr<DirsOptions> {
    fn join_chapters(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>> {
        self.send(JoinChaptersMessage(path))
    }

    fn join_covers(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>> {
        self.send(JoinCoversMessage(path))
    }

    fn join_covers_images(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>> {
        self.send(JoinCoversImagesMessage(path))
    }

    fn join_data(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>> {
        self.send(JoinDataMessage(path))
    }

    fn join_history(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<PathBuf>> {
        self.send(JoinHistoryMessage(path))
    }
}

impl<A> JoinPathAsyncTraits for A
where
    A: GetManagerStateData + Sync,
{
    async fn join_chapters(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<PathBuf> {
        self.get_dir_options().await?.join_chapters(path).await
    }
    async fn join_covers(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<PathBuf> {
        self.get_dir_options().await?.join_covers(path).await
    }
    async fn join_covers_images(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<PathBuf> {
        self.get_dir_options().await?.join_covers_images(path).await
    }
    async fn join_data(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<PathBuf> {
        self.get_dir_options().await?.join_data(path).await
    }
    async fn join_history(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<PathBuf> {
        self.get_dir_options().await?.join_history(path).await
    }
}
