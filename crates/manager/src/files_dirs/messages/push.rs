use std::future::Future;

use actix::prelude::*;
use dev::ToEnvelope;

use crate::{
    data_push::Push, download::state::messages::get::GetManagerStateData, DirsOptions,
    ManagerCoreResult,
};

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
    <Self as Push<T>>::Error: Into<crate::Error>,
{
    type Result = <PushDataMessage<T> as Message>::Result;
    fn handle(&mut self, msg: PushDataMessage<T>, _ctx: &mut Self::Context) -> Self::Result {
        let res = if msg.verify {
            self.verify_and_push(msg.data)
        } else {
            self.push(msg.data)
        };
        res.map_err(|e| e.into())
    }
}

pub trait PushActorAddr<T: Send>: Sync {
    fn push(&self, data: T) -> impl Future<Output = ManagerCoreResult<()>>;
    fn verify_and_push(&self, data: T) -> impl Future<Output = ManagerCoreResult<()>>;
}

impl<T> PushActorAddr<T> for Addr<DirsOptions>
where
    DirsOptions: Push<T> + Handler<PushDataMessage<T>>,
    <DirsOptions as Actor>::Context: ToEnvelope<DirsOptions, PushDataMessage<T>>,
    T: Send + 'static,
{
    async fn push(&self, data: T) -> ManagerCoreResult<()> {
        self.send(PushDataMessage::new(data)).await??;
        Ok(())
    }
    async fn verify_and_push(&self, data: T) -> ManagerCoreResult<()> {
        self.send(PushDataMessage::new(data).verify(true)).await??;
        Ok(())
    }
}

impl<A, T> PushActorAddr<T> for A
where
    A: GetManagerStateData + Sync,
    DirsOptions: Push<T> + Handler<PushDataMessage<T>>,
    <DirsOptions as Actor>::Context: ToEnvelope<DirsOptions, PushDataMessage<T>>,
    T: Send + 'static,
{
    async fn push(&self, data: T) -> ManagerCoreResult<()> {
        self.get_dir_options().await?.push(data).await?;
        Ok(())
    }
    async fn verify_and_push(&self, data: T) -> ManagerCoreResult<()> {
        self.get_dir_options().await?.verify_and_push(data).await?;
        Ok(())
    }
}
