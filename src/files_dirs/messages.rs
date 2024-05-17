pub mod delete;
pub mod join;
pub mod modify;
pub mod pull;
pub mod push;

use super::DirsOptions;
use actix::dev::{MessageResponse, OneshotSender};
use actix::prelude::*;
use std::path::PathBuf;

impl<M> MessageResponse<DirsOptions, M> for PathBuf
where
    M: Message<Result = PathBuf>,
{
    fn handle(
        self,
        _ctx: &mut <super::DirsOptions as actix::Actor>::Context,
        tx: Option<OneshotSender<M::Result>>,
    ) {
        if let Some(tx) = tx {
            let _ = tx.send(self);
        }
    }
}
