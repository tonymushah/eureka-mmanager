use actix::Message;

use crate::{utils::feed::ChapterFeed, feeds::MangaDownloadFeedError};
pub type FeedRtResult = Result<ChapterFeed, MangaDownloadFeedError>;

#[derive(Message)]
#[rtype(result = "FeedRtResult")]
pub struct FeedRtMessage(pub FeedRtResult);