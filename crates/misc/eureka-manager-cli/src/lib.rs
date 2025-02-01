pub mod commands;

use duration_string::DurationString;
use fern::colors::ColoredLevelConfig;
use log::{LevelFilter, Log};
use std::{path::PathBuf, time::SystemTime};

use clap::{Args, Parser};
use commands::Commands;
use eureka_mmanager::prelude::DirsOptionsCore;

#[derive(Debug, Args, Clone)]
pub struct DirsOptionsArgs {
    /// data directory path
    ///
    /// Default: "output"
    #[arg(long)]
    pub data_dir: Option<PathBuf>,
    /// chapter directory relative to `data_dir` (you can put an absolute path if you wanted to)
    #[arg(long)]
    pub chapters: Option<PathBuf>,
    /// manga directory relative to `data_dir` (you can put an absolute path if you wanted to)
    #[arg(long)]
    pub mangas: Option<PathBuf>,
    /// covers directory relative to `data_dir` (you can put an absolute path if you wanted to)
    #[arg(long)]
    pub covers: Option<PathBuf>,
}

impl From<DirsOptionsArgs> for DirsOptionsCore {
    fn from(value: DirsOptionsArgs) -> Self {
        let mut options =
            DirsOptionsCore::new_from_data_dir(value.data_dir.unwrap_or(From::from("output")));
        if let Some(chapters) = value.chapters {
            options.chapters = options.data_dir_add(chapters);
        }
        if let Some(mangas) = value.mangas {
            options.mangas = options.data_dir_add(mangas);
        }
        if let Some(covers) = value.covers {
            options.covers = options.data_dir_add(covers);
        }
        options
    }
}

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, propagate_version = true)]
pub struct Cli {
    /// Verbose
    #[arg(short, long)]
    verbose: bool,
    #[arg(long)]
    pub request_timeout: Option<DurationString>,
    #[command(flatten)]
    pub options: DirsOptionsArgs,
    #[command(subcommand)]
    pub commands: Commands,
}

impl Cli {
    pub fn setup_logger(&self) -> (LevelFilter, Box<dyn Log>) {
        let colors = ColoredLevelConfig::new();
        let (level, log) = fern::Dispatch::new()
            .format(move |out, message, record| {
                out.finish(format_args!(
                    "[{} {} {}] {}",
                    humantime::format_rfc3339_seconds(SystemTime::now()),
                    colors.color(record.level()),
                    record.target(),
                    message
                ));
            })
            .level(if self.verbose {
                LevelFilter::max()
            } else {
                LevelFilter::Error
            })
            .chain(std::io::stdout())
            .into_log();
        (level, log)
    }
}

#[cfg(test)]
mod tests {
    use clap::CommandFactory;

    use crate::Cli;
    #[test]
    fn verify_app() {
        Cli::command().debug_assert();
    }
}
