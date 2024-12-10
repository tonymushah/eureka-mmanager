use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use clap::Args;
use eureka_mmanager::prelude::{DeleteDataAsyncTrait, GetManagerStateData};
use log::info;
use uuid::Uuid;

use crate::commands::AsyncRun;

#[derive(Debug, Args)]
pub struct CoverDeleteArgs {
    /// Cover ids
    pub ids: Vec<Uuid>,
    #[arg(long)]
    pub id_text_file: Vec<PathBuf>,
}

impl CoverDeleteArgs {
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

impl AsyncRun for CoverDeleteArgs {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        let ids = self.get_ids();
        info!("Deleting {} cover", ids.len());
        let dir_option = manager.get_dir_options().await?;
        for id in &ids {
            info!("Deleting cover {}", id);
            dir_option.delete_cover(*id).await?;
            info!("Deleted cover {}", id);
        }
        Ok(())
    }
}
