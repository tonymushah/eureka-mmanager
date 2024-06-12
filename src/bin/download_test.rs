use std::str::FromStr;

use actix::prelude::*;
use mangadex_api::MangaDexClient;
use mangadex_api_types_rust::RelationshipType;
use mangadex_desktop_api2::{
    download::{
        chapter::{task::DownloadMode, ChapterDownloadMessage},
        cover::messages::new_task::CoverDownloadMessage,
        manga::messages::MangaDownloadMessage,
        messages::{
            chapter::GetChapterDownloadManagerMessage, cover::GetCoverDownloadManagerMessage,
            manga::GetMangaDownloadManagerMessage, state::GetManagerStateMessage,
        },
        state::{messages::get::GetDirsOptionsMessage, DownloadMessageState},
        traits::task::AsyncCanBeWaited,
        DownloadManager,
    },
    files_dirs::messages::pull::{
        chapter::ChapterIdsListDataPullMessage,
        cover::CoverListDataPullMessage,
        manga::{MangaDataPullMessage, MangaListDataPullMessage},
    },
    history::{service::messages::is_in::IsInMessage, HistoryEntry},
    DirsOptions,
};
use tokio_stream::StreamExt;
use uuid::Uuid;

use log::{Level, Metadata, Record};
use log::{LevelFilter, SetLoggerError};

struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= Level::Info
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            println!("{} - {}", record.level(), record.args());
        }
    }

    fn flush(&self) {}
}

static LOGGER: SimpleLogger = SimpleLogger;

pub fn init() -> Result<(), SetLoggerError> {
    log::set_logger(&LOGGER).map(|()| log::set_max_level(LevelFilter::Info))
}

fn main() -> anyhow::Result<()> {
    init().map_err(anyhow::Error::msg)?;
    let run = System::new();
    run.block_on(async {
        let chapter_ids = [
            "48ab312a-5cb9-46e9-8061-5eca0dae32e3",
            "8a1906ae-08d3-40ab-8d8f-127cbc940ff1",
            "5ed3d1ba-756e-4077-acbe-4146401c601b",
        ]
        .into_iter()
        .flat_map(Uuid::from_str)
        .collect::<Vec<_>>();
        let dowload_manager = {
            let client = MangaDexClient::default();
            let options = {
                let o = DirsOptions::new_from_data_dir("output");
                o.verify_and_init()?;
                o.start()
            };
            DownloadManager::new(options, client).await.start()
        };
        let options = dowload_manager
            .send(GetManagerStateMessage)
            .await?
            .send(GetDirsOptionsMessage)
            .await?;
        for id in &chapter_ids {
            let chapter = dowload_manager
                .send(GetChapterDownloadManagerMessage)
                .await?
                .send(
                    ChapterDownloadMessage::new(*id)
                        .mode(DownloadMode::DataSaver)
                        .state(DownloadMessageState::Downloading),
                )
                .await?
                .wait()
                .await?
                .await?;
            println!("downloaded chapter [{}]", chapter.id);

            let manga_base: HistoryEntry = chapter
                .find_first_relationships(RelationshipType::Manga)
                .ok_or(anyhow::Error::msg(String::from("Manga not found")))?
                .into();
            if !options.send(IsInMessage(manga_base)).await? {
                let manga = dowload_manager
                    .send(GetMangaDownloadManagerMessage)
                    .await?
                    .send(
                        MangaDownloadMessage::new(manga_base.get_id())
                            .state(DownloadMessageState::Downloading),
                    )
                    .await?
                    .wait()
                    .await?
                    .await?;
                println!("downloaded manga [{}]", manga.id);
                let cover_base: HistoryEntry = manga
                    .find_first_relationships(RelationshipType::CoverArt)
                    .ok_or(anyhow::Error::msg(String::from("CoverArt not found")))?
                    .into();
                if !options.send(IsInMessage(cover_base)).await? {
                    println!("fetching cover");
                    let cover = dowload_manager
                        .send(GetCoverDownloadManagerMessage)
                        .await?
                        .send(
                            CoverDownloadMessage::new(cover_base.get_id())
                                .state(DownloadMessageState::Downloading),
                        )
                        .await?
                        .wait()
                        .await?
                        .await?;
                    println!("download cover [{}]", cover.id);
                }
            } else {
                let cover_base: HistoryEntry = options
                    .send(MangaDataPullMessage(manga_base.get_id()))
                    .await??
                    .find_first_relationships(RelationshipType::CoverArt)
                    .ok_or(anyhow::Error::msg(String::from("CoverArt not found")))?
                    .into();
                if !options.send(IsInMessage(cover_base)).await? {
                    println!("fetching cover");
                    let cover = dowload_manager
                        .send(GetCoverDownloadManagerMessage)
                        .await?
                        .send(
                            CoverDownloadMessage::new(cover_base.get_id())
                                .state(DownloadMessageState::Downloading),
                        )
                        .await?
                        .wait()
                        .await?
                        .await?;
                    println!("download cover [{}]", cover.id);
                }
            }
        }
        let chapter_count = options
            .send(ChapterIdsListDataPullMessage(chapter_ids.clone()))
            .await?
            .fold(0_usize, |acc, _x| acc + 1)
            .await;
        assert_eq!(chapter_count, chapter_ids.len());
        let covers = options
            .send(CoverListDataPullMessage)
            .await??
            .fold(0_usize, |acc, _x| acc + 1)
            .await;
        assert_eq!(covers, 1);
        let mangas = options
            .send(MangaListDataPullMessage)
            .await??
            .fold(0_usize, |acc, _x| acc + 1)
            .await;
        assert_eq!(mangas, 1);
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
