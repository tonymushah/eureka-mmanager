pub mod modify_chapters_path;
pub mod modify_covers_path;
pub mod modify_data_path;
pub mod modify_mangas_path;

use std::{fmt::Debug, future::Future, path::Path};

use actix::Addr;

use crate::{download::state::messages::get::GetManagerStateData, DirsOptions, MailBoxResult};

pub use self::{
    modify_chapters_path::ModifyChaptersDirMessage, modify_covers_path::ModifyCoversDirMessage,
    modify_data_path::ModifyDataDirMessage, modify_mangas_path::ModifyMangaDirMessage,
};

pub trait ModifyDirOptionAsyncTrait: Sync {
    fn modify_chapters_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send;
    fn modify_covers_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send;
    fn modify_data_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send;
    fn modify_mangas_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send;
}

impl ModifyDirOptionAsyncTrait for Addr<DirsOptions> {
    fn modify_chapters_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send {
        self.send(ModifyChaptersDirMessage(path))
    }

    fn modify_covers_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send {
        self.send(ModifyCoversDirMessage(path))
    }

    fn modify_data_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send {
        self.send(ModifyDataDirMessage(path))
    }

    fn modify_mangas_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> impl Future<Output = MailBoxResult<()>> + Send {
        self.send(ModifyMangaDirMessage(path))
    }
}

impl<A> ModifyDirOptionAsyncTrait for A
where
    A: GetManagerStateData + Sync,
{
    async fn modify_chapters_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<()> {
        self.get_dir_options().await?.modify_covers_path(path).await
    }

    async fn modify_covers_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<()> {
        self.get_dir_options().await?.modify_covers_path(path).await
    }

    async fn modify_data_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<()> {
        self.get_dir_options().await?.modify_data_path(path).await
    }

    async fn modify_mangas_path(
        &self,
        path: impl AsRef<Path> + Send + 'static + Debug,
    ) -> MailBoxResult<()> {
        self.get_dir_options().await?.modify_mangas_path(path).await
    }
}
