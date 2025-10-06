pub mod images;

use std::fs::remove_dir_all;

use actix::{Handler, Message};
use images::ChapterImages;
use uuid::Uuid;

use crate::{DirsOptions, files_dirs::events::FilesDirSubscriberMessage};

use self::images::{DeleteChapterImagesError, DeleteChapterImagesMessage};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeleteChapterMessage {
    id: Uuid,
    images: Option<ChapterImages>,
}

impl DeleteChapterMessage {
    pub fn new(id: Uuid) -> Self {
        Self { id, images: None }
    }
    pub fn images(self, images: Option<ChapterImages>) -> Self {
        Self {
            id: self.id,
            images,
        }
    }
}

impl From<DeleteChapterMessage> for Option<DeleteChapterImagesMessage> {
    fn from(value: DeleteChapterMessage) -> Self {
        value
            .images
            .map(|images| DeleteChapterImagesMessage::new(value.id, images))
    }
}

impl Message for DeleteChapterMessage {
    type Result = crate::ManagerCoreResult<()>;
}

impl Handler<DeleteChapterMessage> for DirsOptions {
    type Result = <DeleteChapterMessage as Message>::Result;
    fn handle(&mut self, msg: DeleteChapterMessage, ctx: &mut Self::Context) -> Self::Result {
        let chapter_path = self.chapters_id_add(msg.id);
        if let Some(_msg) = Into::<Option<DeleteChapterImagesMessage>>::into(msg) {
            match self.handle(_msg, ctx) {
                Ok(_) => Ok(()),
                Err(e) => {
                    if let crate::Error::DeleteChapterImages(e) = e {
                        if let DeleteChapterImagesError::Conflict = e {
                            remove_dir_all(chapter_path)?;
                            self.subscribers()
                                .do_send(FilesDirSubscriberMessage::RemovedChapter { id: msg.id });
                            Ok(())
                        } else {
                            Err(crate::Error::DeleteChapterImages(e))
                        }
                    } else {
                        Err(e)
                    }
                }
            }
        } else {
            remove_dir_all(chapter_path)?;
            self.subscribers()
                .do_send(FilesDirSubscriberMessage::RemovedChapter { id: msg.id });
            Ok(())
        }
    }
}
