pub mod cover;
pub mod manga;

use clap::Subcommand;

use super::AsyncRun;

#[derive(Debug, Subcommand)]
pub enum DownloadSubCommands {
    // Download a Manga subcommand
    Manga(manga::MangaDownloadArgs),
    // Download a Cover subcommand
    Cover(cover::CoverDownloadArgs),
}

impl AsyncRun for DownloadSubCommands {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        match self {
            Self::Manga(r) => r.run(manager).await,
            Self::Cover(r) => r.run(manager).await,
        }
    }
}
