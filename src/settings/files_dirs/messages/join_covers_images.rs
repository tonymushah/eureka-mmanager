use crate::settings::files_dirs::DirsOptions;
use actix::prelude::*;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

#[derive(Debug, Message)]
#[rtype(result = "std::path::PathBuf")]
pub struct JoinCoversImagesMessage<T>(T)
where
    T: AsRef<Path> + Debug;

impl<T> Clone for JoinCoversImagesMessage<T>
where
    T: Clone + Debug + AsRef<Path>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<Path> for JoinCoversImagesMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl<T> From<T> for JoinCoversImagesMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Handler<JoinCoversImagesMessage<T>> for DirsOptions
where
    T: AsRef<Path> + Debug,
{
    type Result = PathBuf;
    fn handle(
        &mut self,
        msg: JoinCoversImagesMessage<T>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.cover_images_add(msg)
    }
}
