use actix::prelude::*;
use mangadex_api_types_rust::{CoverSortOrder, OrderDirection};
use mangadex_desktop_api2::{
    files_dirs::messages::pull::cover::CoverListDataPullMessage,
    history::service::HistoryActorService, prelude::*, DirsOptions,
};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("data");
        options.verify_and_init()?;
        let options_actor = options.start();
        let _history = HistoryActorService::new(options_actor.clone()).start();
        let data_pull = options_actor
            .send(CoverListDataPullMessage)
            .await??
            .to_filtered(CoverListDataPullFilterParams {
                manga_ids: [Uuid::parse_str("b4c93297-b32f-4f90-b619-55456a38b0aa")?].into(),
                ..Default::default()
            });
        let data = data_pull
            .to_sorted(CoverSortOrder::Volume(OrderDirection::Descending))
            .await
            .paginate(0, 10)
            .into_results()?;
        for cover in data.data.iter() {
            println!("{}", cover.id);
            println!("{:#?}", cover.attributes);
        }
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
