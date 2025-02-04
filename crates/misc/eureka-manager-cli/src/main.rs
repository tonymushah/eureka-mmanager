use actix::{Actor, System};
use clap::Parser;
use eureka_manager_cli::{
    commands::{AsyncRun, AsyncRunContext},
    Cli,
};
use eureka_mmanager::{
    prelude::{DirsOptions, DirsOptionsCore},
    DownloadManager,
};
use indicatif::MultiProgress;
use indicatif_log_bridge::LogWrapper;
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

fn main() {
    let sys = System::new();
    let progress = MultiProgress::new();
    let cli = Cli::parse();
    let (filter, log) = cli.setup_logger();
    log::set_max_level(filter);
    if let Err(err) = log::set_boxed_logger(Box::new(LogWrapper::new(progress.clone(), log))) {
        eprintln!("{err}");
    }
    let name = clap::crate_name!();
    let version = clap::crate_name!();
    let manager = {
        let options = {
            let dirs_options: DirsOptions = {
                let o: DirsOptionsCore = cli.options.clone().into();
                o.into()
            };
            sys.block_on(async move { dirs_options.start() })
        };
        let mangadex_client = mangadex_api::MangaDexClient::new({
            let mut builder = reqwest::Client::builder().default_headers({
                let mut headers = HeaderMap::new();
                headers.insert(
                    USER_AGENT,
                    HeaderValue::from_str(format!("{name}/{version}").as_str()).unwrap(),
                );
                headers
            });
            if let Some(dura) = cli.request_timeout.as_ref() {
                builder = builder.timeout(**dura);
            }
            builder.build().unwrap()
        });
        sys.block_on(async move { DownloadManager::new(options, mangadex_client).start() })
    };
    let ctx = AsyncRunContext { manager, progress };
    sys.block_on(cli.commands.run(ctx)).unwrap();
}
