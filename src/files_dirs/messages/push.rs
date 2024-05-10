use actix::prelude::*;

use crate::ManagerCoreResult;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct PushDataMessage<T> {
    data: T,
    verify: bool,
}

impl<T> Message for PushDataMessage<T> {
    type Result = ManagerCoreResult<()>;
}

impl<T> PushDataMessage<T> {
    pub fn new(data: T) -> Self {
        Self { data, verify: true }
    }
    pub fn verify(self, verify: bool) -> Self {
        Self { verify, ..self }
    }
}
