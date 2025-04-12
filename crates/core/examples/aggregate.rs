use eureka_mmanager_core::{data_pulls::manga::aggregate::IntoMangaAggreagate, DirsOptions};
use mangadex_api_input_types::manga::aggregate::MangaAggregateParam;
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let opt = DirsOptions::new_from_data_dir("./data");
    let aggregate = opt
        .pull_all_chapter()?
        .flatten()
        .aggregate(MangaAggregateParam {
            manga_id: Uuid::parse_str("5a7e01d5-a31b-4d75-9f50-f31873560a2a")?,
            translated_language: Default::default(),
            groups: Default::default(),
        });
    println!("{:#?}", aggregate);
    Ok(())
}
