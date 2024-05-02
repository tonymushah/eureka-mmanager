use actix::prelude::*;

use crate::{
    history::{
        history_w_file::traits::{AsyncRollBackable, Commitable},
        service::HistoryActorService,
    },
    ManagerCoreResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Message)]
#[rtype(result = "ManagerCoreResult<()>")]
pub struct RollbackMessage;

impl Handler<RollbackMessage> for HistoryActorService {
    type Result = ManagerCoreResult<()>;
    fn handle(&mut self, _msg: RollbackMessage, _ctx: &mut Self::Context) -> Self::Result {
        <Self as Commitable>::commit(self)
    }
}

#[async_trait::async_trait]
impl AsyncRollBackable for Addr<HistoryActorService> {
    type Output = ManagerCoreResult<()>;
    async fn rollback(&mut self) -> <Self as AsyncRollBackable>::Output {
        self.send(RollbackMessage).await?
    }
}
