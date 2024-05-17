use std::fs::remove_dir_all;

use actix::prelude::*;
use uuid::Uuid;

use crate::DirsOptions;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum ChapterImages {
    Data,
    DataSaver,
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteChapterImagesError {
    #[error("The normal images is not found")]
    DataNotFound,
    #[error("The data saver images is not found")]
    DataSaverNotFound,
    #[error("Cannot delete these images, delete the chapter instead")]
    Conflict,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct DeleteChapterImagesMessage {
    id: Uuid,
    images: ChapterImages,
}

impl Message for DeleteChapterImagesMessage {
    type Result = crate::ManagerCoreResult<()>;
}

impl Handler<DeleteChapterImagesMessage> for DirsOptions {
    type Result = <DeleteChapterImagesMessage as Message>::Result;
    fn handle(
        &mut self,
        msg: DeleteChapterImagesMessage,
        _ctx: &mut Self::Context,
    ) -> Self::Result {
        let data_path = self.chapters_id_data_add(msg.id);
        let data_saver_path = self.chapters_id_data_saver_add(msg.id);
        if data_path.exists() && data_saver_path.exists() {
            match msg.images {
                ChapterImages::Data => {
                    remove_dir_all(data_path)?;
                    Ok(())
                }
                ChapterImages::DataSaver => {
                    remove_dir_all(data_saver_path)?;
                    Ok(())
                }
            }
        } else {
            Err(crate::Error::DeleteChapterImages(
                DeleteChapterImagesError::Conflict,
            ))
        }
    }
}
