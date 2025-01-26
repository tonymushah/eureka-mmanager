use std::{fs::File, io::BufReader, path::PathBuf, time::Duration};

use actix::Addr;
use clap::{Args, ValueEnum};
use eureka_mmanager::{
    download::{
        chapter::{task::DownloadMode, ChapterDownloadMessage},
        cover::CoverDownloadMessage,
        manga::MangaDownloadMessage,
        state::DownloadMessageState,
    },
    history::service::messages::is_in::IsInMessage,
    prelude::*,
};
use indicatif::ProgressBar;
use log::{info, trace};
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

use crate::commands::AsyncRun;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Hash, ValueEnum, Default)]
pub enum ChapterDownloadMode {
    /// the default download mode
    #[default]
    Data,
    /// the economic mode
    DataSaver,
}

impl From<ChapterDownloadMode> for DownloadMode {
    fn from(value: ChapterDownloadMode) -> Self {
        match value {
            ChapterDownloadMode::Data => Self::Normal,
            ChapterDownloadMode::DataSaver => Self::DataSaver,
        }
    }
}

#[derive(Debug, Args)]
pub struct ChapterDownloadArgs {
    /// Manga ids
    #[arg(long = "id")]
    pub ids: Vec<Uuid>,
    #[arg(long)]
    pub id_text_file: Vec<PathBuf>,
    #[arg(short, long)]
    pub mode: ChapterDownloadMode,
}

impl ChapterDownloadArgs {
    pub fn get_id_and_modes(&self) -> Vec<(Uuid, ChapterDownloadMode)> {
        let mut res = Vec::new();
        let mut push_res = |value: (Uuid, ChapterDownloadMode)| {
            res.push(value);
        };

        self.ids
            .iter()
            .map(|id| (*id, self.mode))
            .for_each(&mut push_res);

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
            .flat_map(|reader| {
                reader.flat_map(|entry| -> Option<(Uuid, ChapterDownloadMode)> {
                    if entry.contains(';') {
                        let mut split = entry.split(';');
                        let id: Uuid = split.next()?.parse().ok()?;
                        let mode = split
                            .next()
                            .and_then(|part| ChapterDownloadMode::from_str(part, true).ok())
                            .unwrap_or(self.mode);
                        Some((id, mode))
                    } else {
                        Some((entry.parse().ok()?, self.mode))
                    }
                })
            })
            .for_each(&mut push_res);
        res
    }
}

impl AsyncRun for ChapterDownloadArgs {
    async fn run(&self, manager: Addr<DownloadManager>) -> anyhow::Result<()> {
        let ids = self.get_id_and_modes();
        let progress = ProgressBar::new(ids.len() as u64)
            .with_message(format!(
                "Downloading {} chapters with their titles and cover if needed",
                ids.len()
            ))
            .with_elapsed(Duration::from_secs(1));
        trace!(
            "Downloading {} chapters with their titles and cover if needed",
            ids.len()
        );
        for (id, mode) in ids {
            let manager = manager.clone();
            trace!("Downloading chapter {id}");
            let dirs =
                <Addr<DownloadManager> as GetManagerStateData>::get_dir_options(&manager).await?;
            let manga = {
                let chapter_manager =
                    <Addr<DownloadManager> as GetManager<ChapterDownloadManager>>::get(&manager)
                        .await?;
                let mut task = chapter_manager
                    .send(
                        ChapterDownloadMessage::new(id)
                            .state(DownloadMessageState::Downloading)
                            .mode(mode),
                    )
                    .await?;
                let data = task.wait().await?.await?;
                info!(
                    "downloaded chapter {} = {:?}",
                    data.id, data.attributes.title
                );
                data.find_first_relationships(RelationshipType::Manga)
                    .ok_or(anyhow::Error::msg(format!(
                        "Cannot find the chapter {} title",
                        id
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
                let cover = {
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
                    let data = task.wait().await?.await?;
                    info!(
                        "downloaded title {} = {:?}",
                        data.id,
                        data.attributes.title.values().next()
                    );
                    data.find_first_relationships(RelationshipType::CoverArt)
                        .ok_or(anyhow::Error::msg(format!(
                            "Cannot find the title {} cover art",
                            manga.id
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
            progress.inc(1);
        }
        progress.finish();
        Ok(())
    }
}
