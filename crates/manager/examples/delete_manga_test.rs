use std::str::FromStr;

use actix::prelude::*;
use eureka_mmanager::prelude::*;
use tokio_stream::StreamExt;
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    // Init the actix system runner
    let run = System::new();
    run.block_on(async {
        // Start the option actor
        let options_actor = DirsOptions::new_from_data_dir("data").start();
        let manga_id = Uuid::from_str("b4c93297-b32f-4f90-b619-55456a38b0aa")?;
        // You can just call `.delete_manga(Uuid)` to delete a give manga
        let data = options_actor.delete_manga(manga_id).await?;
        // The `MangaDeleteData` consists of `covers` field which is the deleted covers ids
        // and `chapters` field which is the deleted chapters ids
        println!("{data:#?}");
        // Get all the manga chapter
        let chapters: Vec<Uuid> = {
            let params = ChapterListDataPullFilterParams {
                manga_ids: vec![manga_id],
                ..Default::default()
            };
            options_actor
                .get_chapters()
                .await?
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
                .get_covers()
                .await?
                .to_filtered(params)
                .map(|o| o.id)
                .collect()
                .await
        };
        // check if there is no chapters left
        assert!(chapters.is_empty(), "Some chapter still remains");
        // check if there is no covers left
        assert!(covers.is_empty(), "Some covers still remains");
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
