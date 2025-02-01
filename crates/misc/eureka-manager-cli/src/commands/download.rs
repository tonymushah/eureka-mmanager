pub mod chapter;
pub mod cover;
pub mod manga;

use clap::Subcommand;

use super::{AsyncRun, AsyncRunContext};

#[derive(Debug, Subcommand)]
pub enum DownloadSubCommands {
    /// Download Mangas subcommand
    Manga(manga::MangaDownloadArgs),
    /// Download Covers subcommand
    Cover(cover::CoverDownloadArgs),
    /// Download Chapters subcommand
    Chapter(chapter::ChapterDownloadArgs),
}

impl AsyncRun for DownloadSubCommands {
    async fn run(&self, ctx: AsyncRunContext) -> anyhow::Result<()> {
        match self {
            Self::Manga(r) => r.run(ctx).await,
            Self::Cover(r) => r.run(ctx).await,
            Self::Chapter(r) => r.run(ctx).await,
        }
    }
}
