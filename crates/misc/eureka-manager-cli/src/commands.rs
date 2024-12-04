pub mod download;

use clap::Subcommand;
use download::DownloadSubCommands;

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// Download subcommands
    #[command(subcommand)]
    Download(DownloadSubCommands),
}
