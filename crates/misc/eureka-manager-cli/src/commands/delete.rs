pub mod manga;

use clap::Subcommand;

use super::AsyncRun;

#[derive(Debug, Subcommand)]
pub enum DeleteSubcommands {
    Manga(manga::MangaDeleteArgs),
}

impl AsyncRun for DeleteSubcommands {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        match self {
            DeleteSubcommands::Manga(manga_delete_args) => manga_delete_args.run(manager).await,
        }
    }
}
