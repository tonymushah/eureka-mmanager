use crate::files_dirs::DirsOptions;
use actix::prelude::*;
use std::{fmt::Debug, path::Path};

#[derive(Debug, Message)]
#[rtype(result = "()")]
pub struct ModifyDataDirMessage<T>(T)
where
    T: AsRef<Path> + Debug;

impl<T> Clone for ModifyDataDirMessage<T>
where
    T: Clone + Debug + AsRef<Path>,
{
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

impl<T> AsRef<Path> for ModifyDataDirMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn as_ref(&self) -> &Path {
        self.0.as_ref()
    }
}

impl<T> From<T> for ModifyDataDirMessage<T>
where
    T: AsRef<Path> + Debug,
{
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl<T> Handler<ModifyDataDirMessage<T>> for DirsOptions
where
    T: AsRef<Path> + Debug,
{
    type Result = ();
    fn handle(&mut self, msg: ModifyDataDirMessage<T>, _ctx: &mut Self::Context) -> Self::Result {
        self.data_dir = msg.as_ref().to_path_buf();
    }
}
