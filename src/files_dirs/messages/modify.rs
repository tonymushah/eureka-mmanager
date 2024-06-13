pub mod modify_chapters_path;
pub mod modify_covers_path;
pub mod modify_data_path;
pub mod modify_mangas_path;

use std::{fmt::Debug, future::Future, path::Path};

use actix::Addr;

use crate::{DirsOptions, MailBoxResult};

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
