use actix::prelude::*;
use mangadex_desktop_api2::{
    data_pulls::AsyncPaginate,
    files_dirs::messages::chapter_list_data_pull::ChapterListDataPullMessage,
    history::service::HistoryActorService, DirsOptions,
};

fn main() -> anyhow::Result<()> {
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("data");
        options.verify_and_init()?;
        let options_actor = options.start();
        let _history = HistoryActorService::new(options_actor.clone())
            .await
            .start();
        let data_pull = options_actor.send(ChapterListDataPullMessage).await??;
        let data = data_pull.paginate(0, 10).await.into_results()?;
        println!("{:#?}", data);
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
