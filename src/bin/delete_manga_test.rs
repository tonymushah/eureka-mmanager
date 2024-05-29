use std::str::FromStr;

use actix::prelude::*;
use mangadex_desktop_api2::{
    data_pulls::{
        chapter::ChapterListDataPullFilterParams, cover::CoverListDataPullFilterParams,
        IntoParamedFilteredStream,
    },
    files_dirs::messages::{
        delete::DeleteMangaMessage,
        pull::{chapter::ChapterListDataPullMessage, cover::CoverListDataPullMessage},
    },
    DirsOptions,
};
use tokio_stream::StreamExt;
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("data");
        options.verify_and_init()?;
        let options_actor = options.start();
        let manga_id = Uuid::from_str("b4c93297-b32f-4f90-b619-55456a38b0aa")?;
        let data = options_actor.send(DeleteMangaMessage(manga_id)).await??;
        println!("{:#?}", data);
        let chapters: Vec<Uuid> = {
            let params = ChapterListDataPullFilterParams {
                manga_id: Some(manga_id),
                ..Default::default()
            };
            options_actor
                .send(ChapterListDataPullMessage)
                .await??
                .to_filtered(params)
                .map(|o| o.id)
                .collect()
                .await
        };
        let covers: Vec<Uuid> = {
            let params = CoverListDataPullFilterParams {
                manga_ids: [manga_id].into(),
                ..Default::default()
            };
            options_actor
                .send(CoverListDataPullMessage)
                .await??
                .to_filtered(params)
                .map(|o| o.id)
                .collect()
                .await
        };
        println!("{:#?}", chapters);
        println!("{:#?}", covers);
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
