pub mod chapter;
pub mod cover;
pub mod manga;

use clap::{Args, Subcommand, ValueEnum};
use eureka_mmanager::{
    files_dirs::messages::pull::{
        chapter::ChapterListDataPullMessage, cover::CoverListDataPullMessage,
        manga::MangaListDataPullMessage,
    },
    prelude::GetManagerStateData,
};
use mangadex_api_types_rust::{MangaDexDateTime, TagSearchMode};
use serde::{de::IntoDeserializer, Deserialize};

use super::AsyncRun;

#[derive(Debug, Subcommand)]
pub enum CountSubcommand {
    /// Count manga with filters
    Manga(Box<manga::CountMangaArgs>),
    /// Count covers with filters,
    Cover(cover::CountCoverArgs),
    Chapter(Box<chapter::CountChapterArgs>),
}

impl AsyncRun for CountSubcommand {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        match self {
            CountSubcommand::Manga(count_manga_args) => count_manga_args.run(manager).await,
            CountSubcommand::Cover(count_cover_args) => count_cover_args.run(manager).await,
            CountSubcommand::Chapter(count_chapter_args) => count_chapter_args.run(manager).await,
        }
    }
}

#[derive(Debug, Args)]
pub struct CountArgs {
    #[command(subcommand)]
    pub subcommand: Option<CountSubcommand>,
}

impl AsyncRun for CountArgs {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        if let Some(subcommand) = self.subcommand.as_ref() {
            subcommand.run(manager).await
        } else {
            let dirs_options = manager.get_dir_options().await?;
            println!(
                "Number of titles available: {}",
                dirs_options
                    .send(MangaListDataPullMessage)
                    .await??
                    .flatten()
                    .count()
            );
            println!(
                "Number of covers available: {}",
                dirs_options
                    .send(CoverListDataPullMessage)
                    .await??
                    .flatten()
                    .count()
            );
            println!(
                "Number of chapters available: {}",
                dirs_options
                    .send(ChapterListDataPullMessage)
                    .await??
                    .flatten()
                    .count()
            );
            Ok(())
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, ValueEnum)]
pub enum TagSearchModeEnum {
    And,
    Or,
}

impl From<TagSearchModeEnum> for TagSearchMode {
    fn from(value: TagSearchModeEnum) -> Self {
        match value {
            TagSearchModeEnum::And => Self::And,
            TagSearchModeEnum::Or => Self::Or,
        }
    }
}

pub fn mangadex_time_from_str(s: &str) -> Result<MangaDexDateTime, String> {
    MangaDexDateTime::deserialize(s.into_deserializer())
        .map_err(|e: serde::de::value::Error| format!("{e}"))
}
