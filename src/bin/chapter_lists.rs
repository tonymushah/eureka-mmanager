use actix::prelude::*;
use mangadex_api_types_rust::{ChapterSortOrder, Language, OrderDirection};
use mangadex_desktop_api2::{
    data_pulls::{
        chapter::ChapterListDataPullFilterParams, AsyncIntoSorted, IntoParamedFilteredStream,
        Paginate,
    },
    files_dirs::messages::pull::chapter::ChapterListDataPullMessage,
    history::service::HistoryActorService,
    DirsOptions,
};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("data");
        options.verify_and_init()?;
        let options_actor = options.start();
        let _history = HistoryActorService::new(options_actor.clone())
            .await
            .start();
        let data_pull = options_actor
            .send(ChapterListDataPullMessage)
            .await??
            .to_filtered(ChapterListDataPullFilterParams {
                manga_id: Some(Uuid::parse_str("b4c93297-b32f-4f90-b619-55456a38b0aa")?),
                translated_languages: [Language::English].into(),
                ..Default::default()
            });
        let data = data_pull
            .to_sorted(ChapterSortOrder::Chapter(OrderDirection::Ascending))
            .await
            .paginate(0, 10)
            .into_results()?;
        for chapter in data.data.iter() {
            println!("{}", chapter.id);
            println!("{:#?}", chapter.attributes);
        }
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
