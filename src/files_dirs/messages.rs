pub mod join_chapters;
pub mod join_covers;
pub mod join_covers_images;
pub mod join_data;
pub mod join_history;
pub mod modify_chapters_path;
pub mod modify_covers_path;

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
