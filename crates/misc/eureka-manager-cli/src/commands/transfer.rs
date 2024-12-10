use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use clap::Args;
use uuid::Uuid;

use crate::DirsOptionsArgs;

use super::AsyncRun;

#[derive(Debug, Args)]
pub struct TransferCommand {
    /// directory targets for data to transfer in
    #[command(flatten)]
    pub transfer_dirs: DirsOptionsArgs,
    #[command(flatten)]
    pub mangas: TransferMangaArgs,
    #[command(flatten)]
    pub covers: TransferCoverArgs,
    #[command(flatten)]
    pub chapters: TransferChapterArgs,
}

#[derive(Debug, Args)]
pub struct TransferMangaArgs {
    #[arg(long = "manga")]
    pub ids: Vec<Uuid>,
    #[arg(long = "manga-id-text-file")]
    pub id_text_file: Vec<PathBuf>,
}

impl TransferMangaArgs {
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

#[derive(Debug, Args)]
pub struct TransferCoverArgs {
    #[arg(long = "cover")]
    pub ids: Vec<Uuid>,
    #[arg(long = "cover-id-text-file")]
    pub id_text_file: Vec<PathBuf>,
}

impl TransferCoverArgs {
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

#[derive(Debug, Args)]
pub struct TransferChapterArgs {
    #[arg(long = "chapter")]
    pub ids: Vec<Uuid>,
    #[arg(long = "chapter-id-text-file")]
    pub id_text_file: Vec<PathBuf>,
}

impl TransferChapterArgs {
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

impl AsyncRun for TransferCommand {
    async fn run(
        &self,
        manager: actix::Addr<eureka_mmanager::DownloadManager>,
    ) -> anyhow::Result<()> {
        todo!()
    }
}
