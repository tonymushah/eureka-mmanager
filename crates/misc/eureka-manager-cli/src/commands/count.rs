use clap::{Args, Subcommand};
use eureka_mmanager::{
    files_dirs::messages::pull::{
        chapter::ChapterListDataPullMessage, cover::CoverListDataPullMessage,
        manga::MangaListDataPullMessage,
    },
    prelude::GetManagerStateData,
};

use super::AsyncRun;

#[derive(Debug, Subcommand)]
pub enum CountSubcommand {}

impl AsyncRun for CountSubcommand {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        todo!()
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
