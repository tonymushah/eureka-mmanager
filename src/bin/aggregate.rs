use actix::prelude::*;
use mangadex_api_input_types::manga::aggregate::MangaAggregateParam;
use mangadex_api_types_rust::Language;
use mangadex_desktop_api2::{
    data_pulls::manga::aggregate::AsyncIntoMangaAggreagate,
    files_dirs::messages::pull::chapter::ChapterListDataPullMessage,
    history::service::HistoryActorService, DirsOptions,
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
        let aggregate = options_actor
            .send(ChapterListDataPullMessage)
            .await??
            .aggregate(MangaAggregateParam {
                manga_id: Uuid::parse_str("b4c93297-b32f-4f90-b619-55456a38b0aa")?,
                translated_language: [Language::English].into(),
                groups: Default::default(),
            })
            .await;
        println!("{:#?}", aggregate);
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
