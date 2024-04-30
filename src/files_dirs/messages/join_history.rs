use crate::files_dirs::DirsOptions;
use actix::prelude::*;
use std::{
    fmt::Debug,
    path::{Path, PathBuf},
};

#[derive(Debug, Message)]
#[rtype(result = "std::path::PathBuf")]
pub struct JoinHistoryMessage<T>(T)
where
    T: AsRef<Path> + Debug;

impl<T> Clone for JoinHistoryMessage<T>
where
    T: Clone + Debug + AsRef<Path>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<Path> for JoinHistoryMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl<T> From<T> for JoinHistoryMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Handler<JoinHistoryMessage<T>> for DirsOptions
where
    T: AsRef<Path> + Debug,
{
    type Result = PathBuf;
    fn handle(&mut self, msg: JoinHistoryMessage<T>, _ctx: &mut Self::Context) -> Self::Result {
        self.history_add(msg)
    }
}
