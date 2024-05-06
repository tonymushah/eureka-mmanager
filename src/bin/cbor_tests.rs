use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use actix::prelude::*;
use mangadex_desktop_api2::{
    files_dirs::messages::manga_list_data_pull::MangaListDataPullMessage, DirsOptions,
};
use tokio_stream::StreamExt;

fn main() -> anyhow::Result<()> {
    create_dir_all("output/mangas")?;
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("output");
        options.verify_and_init()?;
        let options_actor = options.start();
        let mut data_pull = options_actor.send(MangaListDataPullMessage).await??;
        while let Some(manga) = data_pull.next().await {
            println!("{}", manga.id);
        }
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
