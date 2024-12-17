use clap::Args;
use eureka_mmanager::{
    files_dirs::messages::pull::chapter::ChapterListDataPullMessage,
    prelude::{
        ChapterDataPullAsyncTrait, ChapterListDataPullFilterParams, GetManagerStateData,
        IntoParamedFilteredStream,
    },
};
use mangadex_api_types_rust::{ContentRating, Language, MangaDexDateTime};
use tokio_stream::StreamExt;
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
    /// Show chapter ids
    #[arg(short)]
    pub ids: bool,
}

impl CountChapterArgs {
    fn to_params(&self) -> ChapterListDataPullFilterParams {
        let Self {
            title,
            groups,
            uploaders,
            volumes,
            manga_ids,
            chapters,
            translated_languages,
            original_languages,
            excluded_original_languages,
            content_rating,
            excluded_groups,
            excluded_uploaders,
            created_at_since,
            updated_at_since,
            publish_at_since,
            ..
        } = self;
        ChapterListDataPullFilterParams {
            title: title.clone(),
            groups: groups.clone(),
            uploaders: uploaders.clone(),
            volumes: volumes.clone(),
            manga_ids: manga_ids.clone(),
            chapters: chapters.clone(),
            translated_languages: translated_languages.clone(),
            original_languages: original_languages.clone(),
            excluded_original_languages: excluded_original_languages.clone(),
            content_rating: content_rating.clone(),
            excluded_groups: excluded_groups.clone(),
            excluded_uploaders: excluded_uploaders.clone(),
            created_at_since: *created_at_since,
            updated_at_since: *updated_at_since,
            publish_at_since: *publish_at_since,
        }
    }
}

impl AsyncRun for CountChapterArgs {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        let dir_options = manager.get_dir_options().await?;
        let mut stream = dir_options
            .send(ChapterListDataPullMessage)
            .await??
            .to_filtered(self.to_params());
        if self.ids {
            while let Some(chapter) = stream.next().await {
                println!("{} [{}]", chapter.id, {
                    let images = dir_options.get_chapter_images(chapter.id).await?;
                    match (!images.data.is_empty(), !images.data_saver.is_empty()) {
                        (true, true) => "data;data-saver".to_string(),
                        (true, false) => "data".to_string(),
                        (false, true) => "data-saver".to_string(),
                        (false, false) => String::new(),
                    }
                });
            }
        } else {
            println!(
                "Number of chapters available: {}",
                stream.fold(0usize, |count, _| count + 1).await
            );
        }
        Ok(())
    }
}
