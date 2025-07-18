use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use actix::Actor;
use clap::Args;
use eureka_mmanager::{
    download::chapter::task::DownloadMode,
    prelude::{
        ChapterDataPullAsyncTrait, ChapterImagePushEntry, CoverDataPullAsyncTrait, DirsOptions,
        DirsOptionsCore, GetManagerStateData, MangaDataPullAsyncTrait, PushActorAddr,
    },
};
use log::info;
use mangadex_api_types_rust::RelationshipType;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::DirsOptionsArgs;

use super::{AsyncRun, AsyncRunContext};

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
    #[arg(long)]
    pub verify: bool,
}

#[derive(Debug, Args)]
pub struct TransferMangaArgs {
    #[arg(long)]
    pub manga: Vec<Uuid>,
    #[arg(long)]
    pub manga_ids_text_file: Vec<PathBuf>,
}

impl TransferMangaArgs {
    pub fn get_ids(&self) -> Vec<Uuid> {
        let mut ids = self.manga.clone();
        self.manga_ids_text_file
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
    #[arg(long)]
    pub cover: Vec<Uuid>,
    #[arg(long)]
    pub cover_ids_text_file: Vec<PathBuf>,
}

impl TransferCoverArgs {
    pub fn get_ids(&self) -> Vec<Uuid> {
        let mut ids = self.cover.clone();
        self.cover_ids_text_file
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
    #[arg(long)]
    pub chapter: Vec<Uuid>,
    #[arg(long)]
    pub chapter_ids_text_file: Vec<PathBuf>,
}

impl TransferChapterArgs {
    pub fn get_ids(&self) -> Vec<Uuid> {
        let mut ids = self.chapter.clone();
        self.chapter_ids_text_file
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
    async fn run(&self, ctx: AsyncRunContext) -> anyhow::Result<()> {
        let mut chapters = self.chapters.get_ids();
        let mut mangas = self.mangas.get_ids();
        let mut covers = self.covers.get_ids();

        let target_opts = {
            let opts: DirsOptionsCore = self.transfer_dirs.clone().into();
            let opts: DirsOptions = opts.into();
            opts.start()
        };

        let current_opts = ctx.manager.get_dir_options().await?;
        chapters.dedup();
        let mut chapter_stream = current_opts
            .get_chapters_by_ids(chapters.into_iter())
            .await?;
        // Chapter push
        while let Some(chapter) = StreamExt::next(&mut chapter_stream).await {
            let id = chapter.id;
            let images = current_opts.get_chapter_images(id).await?;
            let images = {
                let mut entries: Vec<ChapterImagePushEntry<BufReader<File>>> = Vec::new();
                for data_entry_filename in images.data.iter() {
                    let entry = ChapterImagePushEntry::new(
                        chapter.id,
                        data_entry_filename.clone(),
                        BufReader::new(
                            current_opts
                                .get_chapter_image(chapter.id, data_entry_filename.clone())
                                .await?,
                        ),
                    )
                    .mode(DownloadMode::Normal);
                    entries.push(entry);
                }
                for data_saver_entry_filename in images.data_saver.iter() {
                    let entry = ChapterImagePushEntry::new(
                        chapter.id,
                        data_saver_entry_filename.clone(),
                        BufReader::new(
                            current_opts
                                .get_chapter_image_data_saver(
                                    chapter.id,
                                    data_saver_entry_filename.clone(),
                                )
                                .await?,
                        ),
                    )
                    .mode(DownloadMode::DataSaver);
                    entries.push(entry);
                }
                entries
            };
            info!("Transfering {id} chapter...");
            let manga = chapter
                .find_first_relationships(RelationshipType::Manga)
                .cloned();

            if self.verify {
                target_opts.verify_and_push(chapter).await?;
                target_opts.verify_and_push(images).await?;
            } else {
                target_opts.push(chapter).await?;
                target_opts.push(images).await?;
            }

            info!("Transfered {id} chapter!");
            if let Some(manga) = manga {
                if mangas.contains(&manga.id) {
                    mangas.push(manga.id);
                }
            }
        }

        mangas.dedup();
        let mut mangas_stream = current_opts
            .get_manga_list_by_ids(mangas.into_iter())
            .await?;
        while let Some(manga) = StreamExt::next(&mut mangas_stream).await {
            let id = manga.id;
            if let Some(cover) = manga
                .find_first_relationships(RelationshipType::CoverArt)
                .cloned()
            {
                covers.push(cover.id);
            }
            info!("Transfering {id} title...");
            if self.verify {
                target_opts.verify_and_push(manga).await?;
            } else {
                target_opts.push(manga).await?;
            }
            info!("Transfered {id} title!");
        }

        covers.dedup();
        let mut cover_stream = current_opts.get_covers_by_ids(covers.into_iter()).await?;
        while let Some(cover) = StreamExt::next(&mut cover_stream).await {
            let id = cover.id;
            info!("Transfering {id} cover...");
            let image = current_opts.get_cover_image(cover.id).await?;
            if self.verify {
                target_opts
                    .verify_and_push((cover, BufReader::new(image)))
                    .await?;
            } else {
                target_opts.push((cover, BufReader::new(image))).await?;
            }
            info!("Transfered {id} cover!");
        }
        Ok(())
    }
}
