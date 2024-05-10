use actix::prelude::*;

use crate::{data_push::Push, DirsOptions, ManagerCoreResult};

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

impl<T> Handler<PushDataMessage<T>> for DirsOptions
where
    Self: Push<T>,
{
    type Result = <PushDataMessage<T> as Message>::Result;
    fn handle(&mut self, msg: PushDataMessage<T>, _ctx: &mut Self::Context) -> Self::Result {
        if msg.verify {
            self.verify_and_push(msg.data)
        } else {
            self.push(msg.data)
        }
    }
}
