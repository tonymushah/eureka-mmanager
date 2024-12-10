use clap::Args;
use eureka_mmanager::prelude::ChapterListDataPullFilterParams;
use mangadex_api_types_rust::{ContentRating, Language, MangaDexDateTime};
use uuid::Uuid;

use crate::commands::AsyncRun;

#[derive(Debug, Args)]
pub struct CountChapterArgs {
    #[arg(long)]
    pub title: Option<String>,
    #[arg(long = "group")]
    pub groups: Vec<Uuid>,
    #[arg(long = "uploader")]
    pub uploaders: Vec<Uuid>,
    #[arg(long = "volume")]
    pub volumes: Vec<String>,
    #[arg(long = "manga")]
    pub manga_ids: Vec<Uuid>,
    /// Chapter number in the series or volume.
    #[arg(long = "chapter")]
    pub chapters: Vec<String>,
    #[arg(long = "translated-language")]
    pub translated_languages: Vec<Language>,
    #[arg(long = "original-language")]
    pub original_languages: Vec<Language>,
    #[arg(long = "excluded-original-language")]
    pub excluded_original_languages: Vec<Language>,
    #[arg(long = "content-rating")]
    pub content_rating: Vec<ContentRating>,
    /// Groups to exclude from the results.
    #[arg(long = "excluded-group")]
    pub excluded_groups: Vec<Uuid>,
    /// Uploaders to exclude from the results.
    #[arg(long = "excluded-uploader")]
    pub excluded_uploaders: Vec<Uuid>,
    #[arg(long, value_parser = crate::commands::count::mangadex_time_from_str)]
    pub created_at_since: Option<MangaDexDateTime>,
    /// DateTime string with following format: `YYYY-MM-DDTHH:MM:SS`.
    #[arg(long, value_parser = crate::commands::count::mangadex_time_from_str)]
    pub updated_at_since: Option<MangaDexDateTime>,
    /// DateTime string with following format: `YYYY-MM-DDTHH:MM:SS`.
    #[arg(long, value_parser = crate::commands::count::mangadex_time_from_str)]
    pub publish_at_since: Option<MangaDexDateTime>,
}

impl AsyncRun for CountChapterArgs {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
