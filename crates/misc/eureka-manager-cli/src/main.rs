use actix::{Actor, System};
use clap::Parser;
use eureka_manager_cli::{commands::AsyncRun, Cli};
use eureka_mmanager::{
    prelude::{DirsOptions, DirsOptionsCore},
    DownloadManager,
};
use reqwest::header::{HeaderMap, HeaderValue, USER_AGENT};

fn main() {
    let sys = System::new();

    let cli = Cli::parse();
    cli.setup_logger().unwrap();
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
        let mangadex_client = mangadex_api::MangaDexClient::new(
            reqwest::Client::builder()
                .default_headers({
                    let mut headers = HeaderMap::new();
                    headers.insert(
                        USER_AGENT,
                        HeaderValue::from_str(format!("{name}/{version}").as_str()).unwrap(),
                    );
                    headers
                })
                .build()
                .unwrap(),
        );
        sys.block_on(async move { DownloadManager::new(options, mangadex_client).start() })
    };
    sys.block_on(cli.commands.run(manager)).unwrap();
}
