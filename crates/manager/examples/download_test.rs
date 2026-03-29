use std::num::NonZero;

use actix::prelude::*;
use clap::Parser;
use eureka_mmanager::{
    DirsOptions,
    download::{
        DownloadManager,
        chapter::{ChapterDownloadMessage, task::DownloadMode},
        cover::CoverDownloadMessage,
        manga::MangaDownloadMessage,
        messages::{
            chapter::GetChapterDownloadManagerMessage, cover::GetCoverDownloadManagerMessage,
            manga::GetMangaDownloadManagerMessage, state::GetManagerStateMessage,
        },
        state::{DownloadMessageState, messages::get::GetDirsOptionsMessage},
        traits::task::AsyncCanBeWaited,
    },
    files_dirs::messages::pull::{
        chapter::ChapterIdsListDataPullMessage,
        cover::CoverListDataPullMessage,
        manga::{MangaDataPullMessage, MangaListDataPullMessage},
    },
    history::{HistoryEntry, service::messages::is_in::IsInMessage},
};
use log::debug;
use mangadex_api::MangaDexClient;
use mangadex_api_types_rust::RelationshipType;
use tokio::task::JoinSet;
use uuid::Uuid;

#[derive(Debug, Parser)]
#[command(version, about, long_about = None, propagate_version = true)]
struct Cli {
    expected_mangas: Option<NonZero<u8>>,
    expected_covers: Option<NonZero<u8>>,
    chapters: Vec<Uuid>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    env_logger::init();
    let run = System::with_tokio_rt(|| {
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap()
    });
    run.block_on(async {
        if tokio::runtime::Handle::try_current().is_ok() {
            log::info!("Has a tokio handle! :D");
        }
        let chapter_ids = cli.chapters;
        let dowload_manager = {
            let client = MangaDexClient::default();
            let options = {
                let o = DirsOptions::new_from_data_dir("output");
                o.verify_and_init()?;
                o.start()
            };
            DownloadManager::new(options, client).start()
        };
        let options = dowload_manager
            .send(GetManagerStateMessage)
            .await?
            .send(GetDirsOptionsMessage)
            .await?;
        let mut join_set = JoinSet::<Result<(), anyhow::Error>>::new();
        for id in chapter_ids.iter().cloned() {
            let dw_mangr = dowload_manager.clone();
            let options = options.clone();
            join_set.spawn(async move {
                log::info!("downloading chapter {id}");
                let chapter = {
                    let c_manager = dw_mangr.send(GetChapterDownloadManagerMessage).await?;
                    debug!("Got manager");

                    let mut task = c_manager
                        .send(
                            ChapterDownloadMessage::new(id)
                                .mode(DownloadMode::DataSaver)
                                .state(DownloadMessageState::Downloading),
                        )
                        .await?;
                    debug!("Got task!");
                    let wait = task.wait().await?;
                    debug!("Got wait");
                    let res = wait.await?;
                    debug!("Downloaded chapter {id}");
                    res
                };
                println!("downloaded chapter [{}]", chapter.id);

                let manga_base: HistoryEntry = chapter
                    .find_first_relationships(RelationshipType::Manga)
                    .ok_or(anyhow::Error::msg(String::from("Manga not found")))?
                    .into();
                if !options.send(IsInMessage(manga_base)).await? {
                    let manga = dw_mangr
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
                        let cover = dw_mangr
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
                        let cover = dw_mangr
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
                log::info!("task done!");
                Ok(())
            });
        }
        while let Some(res) = join_set.join_next().await {
            res??;
        }
        let chapter_count = options
            .send(ChapterIdsListDataPullMessage(chapter_ids.clone()))
            .await?
            .flatten()
            .fold(0_usize, |acc, _x| acc + 1);
        assert_eq!(chapter_count, chapter_ids.len());
        let covers = options
            .send(CoverListDataPullMessage)
            .await??
            .flatten()
            .fold(0_usize, |acc, _x| acc + 1);
        assert_eq!(
            covers,
            cli.expected_covers.unwrap_or(1.try_into()?).get() as usize
        );
        let mangas = options
            .send(MangaListDataPullMessage)
            .await??
            .flatten()
            .fold(0_usize, |acc, _x| acc + 1);
        assert_eq!(
            mangas,
            cli.expected_mangas.unwrap_or(1.try_into()?).get() as usize
        );
        Ok::<(), anyhow::Error>(())
    })?;
    Ok(())
}
