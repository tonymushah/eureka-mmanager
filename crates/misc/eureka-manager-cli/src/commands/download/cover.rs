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
use mangadex_api::v5::schema::RelatedAttributes;
use mangadex_api_types_rust::RelationshipType;
use tokio::task::JoinSet;
use uuid::Uuid;

use crate::commands::AsyncRun;

#[derive(Debug, Args)]
pub struct CoverDownloadArgs {
    /// Manga ids
    #[arg(long = "id")]
    pub ids: Vec<Uuid>,
    #[arg(long)]
    pub id_text_file: Vec<PathBuf>,
}

impl CoverDownloadArgs {
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

impl AsyncRun for CoverDownloadArgs {
    async fn run(&self, manager: Addr<DownloadManager>) -> anyhow::Result<()> {
        let mut tasks = JoinSet::<anyhow::Result<()>>::new();
        let ids = self.get_ids();
        trace!(
            "Downloading {} covers with their titles if missing",
            ids.len()
        );
        for id in ids {
            let manager = manager.clone();
            tasks.spawn(async move {
                trace!("Downloading Cover {id}");
                let dirs =
                    <Addr<DownloadManager> as GetManagerStateData>::get_dir_options(&manager)
                        .await?;
                let manga = {
                    let manga_manager =
                        <Addr<DownloadManager> as GetManager<CoverDownloadManager>>::get(&manager)
                            .await?;
                    let mut task = manga_manager
                        .send(
                            CoverDownloadMessage::new(id).state(DownloadMessageState::Downloading),
                        )
                        .await?;
                    let data = task.wait().await?.await?;
                    info!(
                        "downloaded cover {} = {:?}",
                        data.id, data.attributes.file_name
                    );
                    data.find_first_relationships(RelationshipType::Manga)
                        .ok_or(anyhow::Error::msg(format!(
                            "Cannot find the title for cover art {}",
                            id,
                        )))?
                        .clone()
                };
                if !dirs
                    .send(IsInMessage(HistoryEntry::new(
                        manga.id,
                        RelationshipType::Manga,
                    )))
                    .await?
                {
                    trace!("Downloading title {}", manga.id);
                    let manga_manager =
                        <Addr<DownloadManager> as GetManager<MangaDownloadManager>>::get(&manager)
                            .await?;
                    let mut task = manga_manager
                        .send(
                            MangaDownloadMessage::new(manga.id)
                                .state(DownloadMessageState::Downloading),
                        )
                        .await?;
                    task.wait().await?.await?;
                    info!(
                        "downloaded title {} = {:?}",
                        manga.id,
                        manga.attributes.and_then(|attr| {
                            let RelatedAttributes::Manga(manga) = attr else {
                                return None;
                            };
                            manga.title.values().next().cloned()
                        })
                    );
                }
                Ok(())
            });
        }
        while let Some(task) = tasks.join_next().await {
            task??;
        }
        Ok(())
    }
}
