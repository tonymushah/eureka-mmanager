pub mod download;

use std::future::Future;

use actix::Addr;
use clap::Subcommand;
use download::DownloadSubCommands;
use eureka_mmanager::DownloadManager;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download subcommands
    #[command(subcommand)]
    Download(DownloadSubCommands),
}

pub trait AsyncRun: Sync {
    fn run(
        &self,
        manager: Addr<DownloadManager>,
    ) -> impl Future<Output = anyhow::Result<()>> + Send;
}

impl AsyncRun for Commands {
    async fn run(&self, manager: Addr<DownloadManager>) -> anyhow::Result<()> {
        match self {
            Self::Download(d) => d.run(manager).await,
        }
    }
}
