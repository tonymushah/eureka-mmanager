use clap::{Args, ValueEnum};
use eureka_mmanager::{
    files_dirs::messages::pull::manga::MangaListDataPullMessage,
    prelude::{GetManagerStateData, IntoParamedFilteredStream, MangaListDataPullFilterParams},
};
use mangadex_api_types_rust::{
    ContentRating, Demographic, Language, MangaDexDateTime, MangaStatus,
};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::commands::{AsyncRun, AsyncRunContext};

use super::TagSearchModeEnum;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
pub enum MangaStatusEnum {
    /// Manga is still going on.
    Ongoing,
    /// Manga is completed.
    Completed,
    /// Manga is paused from publishing new chapters.
    Hiatus,
    /// Manga has been cancelled.
    Cancelled,
}

impl From<MangaStatusEnum> for MangaStatus {
    fn from(value: MangaStatusEnum) -> Self {
        match value {
            MangaStatusEnum::Ongoing => Self::Ongoing,
            MangaStatusEnum::Completed => Self::Completed,
            MangaStatusEnum::Hiatus => Self::Hiatus,
            MangaStatusEnum::Cancelled => Self::Cancelled,
        }
    }
}

#[derive(Debug, Args)]
pub struct CountMangaArgs {
    #[arg(short, long)]
    pub title: Option<String>,
    #[arg(long)]
    pub author_or_artist: Option<Uuid>,
    #[arg(long = "author")]
    pub authors: Vec<Uuid>,
    #[arg(long = "artist")]
    pub artists: Vec<Uuid>,
    #[arg(short, long)]
    pub year: Option<u16>,
    #[arg(long)]
    pub included_tags: Vec<Uuid>,
    #[arg(long)]
    pub included_tags_mode: Option<TagSearchModeEnum>,
    #[arg(long)]
    pub excluded_tags: Vec<Uuid>,
    #[arg(long)]
    pub excluded_tags_mode: Option<TagSearchModeEnum>,
    #[arg(long)]
    pub status: Vec<MangaStatusEnum>,
    #[arg(long)]
    pub original_language: Vec<Language>,
    #[arg(long)]
    pub excluded_original_language: Vec<Language>,
    #[arg(long)]
    pub publication_demographic: Vec<Demographic>,
    #[arg(long)]
    pub content_rating: Vec<ContentRating>,
    #[arg(long, value_parser = super::mangadex_time_from_str)]
    pub created_at_since: Option<MangaDexDateTime>,
    #[arg(long, value_parser = super::mangadex_time_from_str)]
    pub updated_at_since: Option<MangaDexDateTime>,
    #[arg(long)]
    pub group: Option<Uuid>,
    /// Shows ids instead of the number
    #[arg(long)]
    pub ids: bool,
}

impl CountMangaArgs {
    fn filtered_params(&self) -> MangaListDataPullFilterParams {
        MangaListDataPullFilterParams {
            title: self.title.clone(),
            author_or_artist: self.author_or_artist,
            authors: self.authors.clone(),
            artists: self.artists.clone(),
            year: self.year,
            included_tags: self.included_tags.clone(),
            included_tags_mode: self.included_tags_mode.map(|m| m.into()),
            excluded_tags: self.excluded_tags.clone(),
            excluded_tags_mode: self.excluded_tags_mode.map(|m| m.into()),
            status: self.status.iter().map(|s| (*s).into()).collect(),
            original_language: self.original_language.clone(),
            excluded_original_language: self.excluded_original_language.clone(),
            publication_demographic: self.publication_demographic.clone(),
            content_rating: self.content_rating.clone(),
            created_at_since: self.created_at_since,
            updated_at_since: self.updated_at_since,
            group: self.group,
        }
    }
}

impl AsyncRun for CountMangaArgs {
    async fn run(&self, ctx: AsyncRunContext) -> anyhow::Result<()> {
        let dir_options = ctx.manager.get_dir_options().await?;
        let mut stream = dir_options
            .send(MangaListDataPullMessage)
            .await??
            .to_filtered(self.filtered_params());
        if self.ids {
            while let Some(manga) = stream.next().await {
                println!("{}", manga.id);
            }
        } else {
            println!(
                "Number of title available: {}",
                stream.fold(0usize, |count, _| count + 1).await
            );
        }
        Ok(())
    }
}
