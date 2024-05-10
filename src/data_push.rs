pub mod chapter;

use std::ops::{Deref, DerefMut};

use crate::{DirsOptions, ManagerCoreResult};

#[derive(Debug)]
pub struct DataPush<'a>(&'a mut DirsOptions);

impl<'a> Deref for DataPush<'a> {
    type Target = DirsOptions;
    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl<'a> DerefMut for DataPush<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.0
    }
}

impl DirsOptions {
    pub fn data_push(&mut self) -> DataPush<'_> {
        DataPush(self)
    }
}

pub trait Push<T> {
    fn push(&mut self, data: T) -> ManagerCoreResult<()>;
}
