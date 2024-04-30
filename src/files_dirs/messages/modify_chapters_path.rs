use crate::files_dirs::DirsOptions;
use actix::prelude::*;
use std::{fmt::Debug, path::Path};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ModifyChaptersDirMessage<T>(T)
where
    T: AsRef<Path> + Debug;

impl<T> Clone for ModifyChaptersDirMessage<T>
where
    T: Clone + Debug + AsRef<Path>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<Path> for ModifyChaptersDirMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl<T> From<T> for ModifyChaptersDirMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Handler<ModifyChaptersDirMessage<T>> for DirsOptions
where
    T: AsRef<Path> + Debug,
{
    type Result = ();
    fn handle(
        &mut self,
        msg: ModifyChaptersDirMessage<T>,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        self.chapters = msg.as_ref().to_path_buf();
    }
}
