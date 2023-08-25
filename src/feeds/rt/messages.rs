use actix::Message;

use crate::{feeds::MangaDownloadFeedError, utils::feed::ChapterFeed};
pub type FeedRtResult = Result<ChapterFeed, MangaDownloadFeedError>;

#[derive(Message)]
#[rtype(result = "()")]
pub struct FeedRtMessage(pub FeedRtResult);
