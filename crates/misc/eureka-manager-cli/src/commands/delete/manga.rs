use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use clap::Args;
use eureka_mmanager::prelude::{DeleteDataAsyncTrait, GetManagerStateData};
use log::info;
use uuid::Uuid;

use crate::commands::{AsyncRun, AsyncRunContext};

#[derive(Debug, Args)]
pub struct MangaDeleteArgs {
    /// Manga ids
    pub ids: Vec<Uuid>,
    #[arg(long)]
    pub id_text_file: Vec<PathBuf>,
}

impl MangaDeleteArgs {
    pub fn get_ids(&self) -> Vec<Uuid> {
        let mut ids = self.ids.clone();
        self.id_text_file
            .iter()
            .map(|e| (e, File::open(e)))
            .flat_map(|(path, res)| match res {
                Ok(file) => Some(id_list_txt_reader::IdListTxtReader::new(BufReader::new(
                    file,
                ))),
                Err(err) => {
                    log::error!("Cannot open the {} file: {}", path.to_string_lossy(), err);
                    None
                }
            })
            .flat_map(|file| file.flat_map(|s| Uuid::from_str(&s)))
            .for_each(|id| {
                ids.push(id);
            });
        ids.dedup();
        ids
    }
}

impl AsyncRun for MangaDeleteArgs {
    async fn run(&self, ctx: AsyncRunContext) -> anyhow::Result<()> {
        let ids = self.get_ids();
        info!("Deleting {} titles", ids.len());
        let dir_option = ctx.manager.get_dir_options().await?;
        for id in &ids {
            info!("Deleting title {}", id);
            let delete_data = dir_option.delete_manga(*id).await?;
            info!("Deleted title {}", id);
            info!(
                "Deleted {} covers: {:?}",
                delete_data.covers.len(),
                delete_data.covers
            );
            info!(
                "Deleted {} chapters: {:?}",
                delete_data.chapters.len(),
                delete_data.chapters
            );
        }
        Ok(())
    }
}
