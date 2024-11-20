use actix::prelude::*;
use mangadex_api_types_rust::{MangaSortOrder, OrderDirection};
use mangadex_desktop_api2::prelude::*;

fn main() -> anyhow::Result<()> {
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("data");
        options.verify_and_init()?;
        let options_actor = options.start();
        let history = HistoryActorService::new(options_actor.clone()).start();
        let params = MangaListDataPullFilterParams {
            ..Default::default()
        };
        println!("{:#?}", params);
        let data_pull = options_actor
            .get_manga_list()
            .await?
            .to_filtered(params)
            .to_sorted(MangaSortOrder::Year(OrderDirection::Ascending))
            .await;
        for manga in data_pull.iter() {
            let has_failed = history
                .is_in(HistoryEntry::new(manga.id, manga.type_))
                .await?;
            println!("{:#?} - {has_failed}", manga.id);
            if let Some(year) = manga.attributes.year {
                println!("year {year}",)
            }
        }
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
