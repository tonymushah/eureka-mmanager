use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use actix::Addr;
use clap::Args;
use eureka_mmanager::{
    download::{
        cover::CoverDownloadMessage, manga::MangaDownloadMessage, state::DownloadMessageState,
    },
    history::service::messages::is_in::IsInMessage,
    prelude::*,
};
use log::{info, trace};
use mangadex_api_types_rust::RelationshipType;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::commands::AsyncRun;

#[derive(Debug, Args)]
pub struct MangaDownloadArgs {
    /// Manga ids
    #[arg(long = "id")]
    pub ids: Vec<Uuid>,
    #[arg(long)]
    pub id_text_file: Vec<PathBuf>,
}

impl MangaDownloadArgs {
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

impl AsyncRun for MangaDownloadArgs {
    async fn run(&self, manager: Addr<DownloadManager>) -> anyhow::Result<()> {
        let ids = self.get_ids();
        trace!("Downloading {} titles with their cover", ids.len());
        for id in ids {
            let manager = manager.clone();
            trace!("Downloading title {id}");
            let dirs =
                <Addr<DownloadManager> as GetManagerStateData>::get_dir_options(&manager).await?;
            let cover = {
                let manga_manager =
                    <Addr<DownloadManager> as GetManager<MangaDownloadManager>>::get(&manager)
                        .await?;
                let mut task = manga_manager
                    .send(MangaDownloadMessage::new(id).state(DownloadMessageState::Downloading))
                    .await?;
                let data = task.wait().await?.await?;
                info!(
                    "downloaded title {} = {:?}",
                    data.id,
                    data.attributes.title.values().next()
                );
                data.find_first_relationships(RelationshipType::CoverArt)
                    .ok_or(anyhow::Error::msg(format!(
                        "Cannot find the title {} cover art",
                        id
                    )))?
                    .clone()
            };
            if !dirs
                .send(IsInMessage(HistoryEntry::new(
                    cover.id,
                    RelationshipType::CoverArt,
                )))
                .await?
            {
                trace!("Downloading {} cover art", cover.id);
                let cover_manager =
                    <Addr<DownloadManager> as GetManager<CoverDownloadManager>>::get(&manager)
                        .await?;
                let mut task = cover_manager
                    .send(
                        CoverDownloadMessage::new(cover.id)
                            .state(DownloadMessageState::Downloading),
                    )
                    .await?;
                task.wait().await?.await?;
                info!("Downloaded {} cover art", cover.id);
            }
        }
        Ok(())
    }
}
