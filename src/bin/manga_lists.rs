use actix::prelude::*;
use mangadex_desktop_api2::{
    files_dirs::messages::pull::manga::MangaListDataPullMessage,
    history::{
        service::{messages::is_in::IsInMessage, HistoryActorService},
        HistoryEntry,
    },
    DirsOptions,
};
use tokio_stream::StreamExt;

fn main() -> anyhow::Result<()> {
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("data");
        options.verify_and_init()?;
        let options_actor = options.start();
        let history = HistoryActorService::new(options_actor.clone())
            .await
            .start();
        let mut data_pull = options_actor.send(MangaListDataPullMessage).await??;
        while let Some(manga) = data_pull.next().await {
            let has_failed = history
                .send(IsInMessage(HistoryEntry::new(manga.id, manga.type_)))
                .await?;
            println!("{:#?} - {has_failed}", manga.id);
        }
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
