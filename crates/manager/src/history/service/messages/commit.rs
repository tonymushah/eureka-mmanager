use actix::prelude::*;

use crate::{
    history::{
        history_w_file::traits::{AsyncCommitable, Commitable},
        service::HistoryActorService,
    },
    ManagerCoreResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Default, Message)]
#[rtype(result = "ManagerCoreResult<()>")]
pub struct CommitMessage;

impl Handler<CommitMessage> for HistoryActorService {
    type Result = ManagerCoreResult<()>;
    fn handle(&mut self, _msg: CommitMessage, _ctx: &mut Self::Context) -> Self::Result {
        <Self as Commitable>::commit(self)
    }
}

impl AsyncCommitable for Addr<HistoryActorService> {
    type Output = ManagerCoreResult<()>;
    async fn commit(&self) -> <Self as AsyncCommitable>::Output {
        self.send(CommitMessage).await?
    }
}
