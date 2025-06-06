pub mod count;
pub mod delete;
pub mod download;
pub mod transfer;

use std::future::Future;

use actix::Addr;
use clap::Subcommand;
use download::DownloadSubCommands;
use eureka_mmanager::DownloadManager;
use indicatif::MultiProgress;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download subcommands
    #[command(subcommand)]
    Download(DownloadSubCommands),
    Count(Box<count::CountArgs>),
    #[command(subcommand)]
    Remove(delete::DeleteSubcommands),
    Transfert(Box<transfer::TransferCommand>),
}

#[derive(Debug, Clone)]
pub struct AsyncRunContext {
    pub manager: Addr<DownloadManager>,
    pub progress: MultiProgress,
}

pub trait AsyncRun: Sync {
    fn run(&self, ctx: AsyncRunContext) -> impl Future<Output = anyhow::Result<()>> + Send;
}

impl AsyncRun for Commands {
    async fn run(&self, manager: AsyncRunContext) -> anyhow::Result<()> {
        match self {
            Commands::Download(download_sub_commands) => download_sub_commands.run(manager).await,
            Commands::Count(count_args) => count_args.run(manager).await,
            Commands::Remove(delete_subcommands) => delete_subcommands.run(manager).await,
            Commands::Transfert(transfer_command) => transfer_command.run(manager).await,
        }
    }
}
