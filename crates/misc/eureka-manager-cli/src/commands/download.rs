pub mod chapter;
pub mod cover;
pub mod manga;

use clap::Subcommand;

use super::AsyncRun;

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
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Manga(r) => r.run(manager).await,
            Self::Cover(r) => r.run(manager).await,
            Self::Chapter(r) => r.run(manager).await,
        }
    }
}
