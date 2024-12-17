use std::{
    fs::{create_dir_all, File},
    io::{BufWriter, Write},
};

use actix::prelude::*;
use eureka_mmanager::{files_dirs::messages::pull::manga::MangaListDataPullMessage, DirsOptions};
use tokio_stream::StreamExt;

fn main() -> anyhow::Result<()> {
    create_dir_all("output/mangas")?;
    let run = System::new();
    run.block_on(async {
        let options = DirsOptions::new_from_data_dir("data");
        options.verify_and_init()?;
        let options_actor = options.start();
        let mut data_pull = options_actor.send(MangaListDataPullMessage).await??;
        while let Some(manga) = StreamExt::next(&mut data_pull).await {
            println!("{}", manga.id);
            let mut file = BufWriter::new(File::create(format!("output/manga/{}.cbor", manga.id))?);
            ciborium::into_writer(&manga, &mut file)?;
            file.flush()?;
        }
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
