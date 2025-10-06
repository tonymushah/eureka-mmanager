use std::fs::remove_file;

use actix::prelude::*;
use uuid::Uuid;

use crate::{
    DirsOptions,
    files_dirs::{events::FilesDirSubscriberMessage, messages::pull::cover::CoverDataPullMessage},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeleteCoverMessage(pub Uuid);

impl Message for DeleteCoverMessage {
    type Result = crate::ManagerCoreResult<()>;
}

impl From<Uuid> for DeleteCoverMessage {
    fn from(value: Uuid) -> Self {
        Self(value)
    }
}

impl From<DeleteCoverMessage> for Uuid {
    fn from(value: DeleteCoverMessage) -> Self {
        value.0
    }
}

impl Handler<DeleteCoverMessage> for DirsOptions {
    type Result = <DeleteCoverMessage as Message>::Result;
    fn handle(&mut self, msg: DeleteCoverMessage, ctx: &mut Self::Context) -> Self::Result {
        let cover = self.handle(CoverDataPullMessage(msg.into()), ctx)?;
        let image_path = self.cover_images_add(cover.attributes.file_name);
        remove_file(image_path)?;
        remove_file(self.covers_add(format!("{}.json", msg.0)))?;
        self.subscribers()
            .do_send(FilesDirSubscriberMessage::RemovedCoverArt { id: msg.0 });
        Ok(())
    }
}
