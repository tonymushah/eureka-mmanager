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

pub trait Push<T> {
    fn push(&mut self, data: T) -> ManagerCoreResult<()>;
}

impl<I, D, T> Push<I> for T
where
    T: Push<D>,
    I: Iterator<Item = D>,
{
    fn push(&mut self, data: I) -> ManagerCoreResult<()> {
        for item in data {
            self.push(item)?;
        }
        Ok(())
    }
}
