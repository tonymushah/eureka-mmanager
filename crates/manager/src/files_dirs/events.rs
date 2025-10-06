use actix::Message;
use uuid::Uuid;

use crate::files_dirs::messages::delete::chapter::images::ChapterImages;

#[derive(Debug, Clone, Message)]
#[non_exhaustive]
#[rtype("()")]
pub enum FilesDirSubscriberMessage {
    RemovedManga {
        id: Uuid,
    },
    RemovedCoverArt {
        id: Uuid,
    },
    RemovedChapter {
        id: Uuid,
    },
    RemovedChapterImages {
        id: Uuid,
        mode: Option<ChapterImages>,
    },
}
