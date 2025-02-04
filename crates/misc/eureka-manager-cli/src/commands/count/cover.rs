use clap::Args;
use eureka_mmanager::{
    files_dirs::messages::pull::cover::CoverListDataPullMessage,
    prelude::{
        CoverListDataPullFilterParams, GetManagerStateData, IntoParamedFilteredStream,
        JoinPathAsyncTraits,
    },
};
use mangadex_api_types_rust::Language;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::commands::{AsyncRun, AsyncRunContext};

#[derive(Debug, Args)]
pub struct CountCoverArgs {
    #[arg(long = "manga")]
    pub manga_ids: Vec<Uuid>,
    #[arg(long = "uploader")]
    pub uploader_ids: Vec<Uuid>,
    #[arg(long = "local")]
    pub locales: Vec<Language>,
    #[arg(short, long)]
    pub ids: bool,
    #[arg(short, long)]
    pub filename: bool,
}

impl CountCoverArgs {
    fn to_params(&self) -> CoverListDataPullFilterParams {
        CoverListDataPullFilterParams {
            manga_ids: self.manga_ids.clone(),
            uploader_ids: self.uploader_ids.clone(),
            locales: self.locales.clone(),
        }
    }
}

impl AsyncRun for CountCoverArgs {
    async fn run(&self, ctx: AsyncRunContext) -> anyhow::Result<()> {
        let dir_options = ctx.manager.get_dir_options().await?;
        let mut stream = dir_options
            .send(CoverListDataPullMessage)
            .await??
            .to_filtered(self.to_params());

        match (self.ids, self.filename) {
            (true, false) => {
                while let Some(cover) = stream.next().await {
                    println!("{}", cover.id);
                }
            }
            (true, true) => {
                while let Some(cover) = stream.next().await {
                    println!(
                        "{} [{}]",
                        cover.id,
                        dir_options
                            .join_covers_images(cover.attributes.file_name.clone())
                            .await
                            .ok()
                            .and_then(|p| p.to_str().map(String::from))
                            .unwrap_or(cover.attributes.file_name)
                    );
                }
            }
            (false, true) => {
                while let Some(cover) = stream.next().await {
                    println!(
                        "[{}]",
                        dir_options
                            .join_covers_images(cover.attributes.file_name.clone())
                            .await
                            .ok()
                            .and_then(|p| p.to_str().map(String::from))
                            .unwrap_or(cover.attributes.file_name)
                    );
                }
            }
            (false, false) => {
                println!(
                    "Number of cover art available: {}",
                    stream.fold(0usize, |count, _| count + 1).await
                );
            }
        }
        Ok(())
    }
}
