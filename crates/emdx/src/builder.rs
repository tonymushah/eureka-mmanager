mod inner;

use std::{
    io::{self, Write},
    ops::Deref,
    path::PathBuf,
};

use api_core::{
    data_pulls::{
        chapter::images::ChapterImagesData, cover::CoverListDataPullFilterParams, IntoFiltered,
        Pull, Rand,
    },
    data_push::chapter::image::Mode as ChapterImagesMode,
    DirsOptions,
};
use inner::BuilderInner;
use mangadex_api_schema_rust::v5::{ChapterObject, CoverObject, MangaObject};
use mangadex_api_types_rust::RelationshipType;

use tar::HeaderMode;
use uuid::Uuid;

use crate::{PMangaObject, PackageContents, ThisResult};

#[derive(Debug, Clone)]
pub struct Builder {
    initial_dir_options: DirsOptions,
    contents: PackageContents,
    compression_level: i32,
    compress_image_to_jpeg: bool,
    header_mode: HeaderMode,
}

impl Default for Builder {
    fn default() -> Self {
        Self {
            initial_dir_options: Default::default(),
            contents: Default::default(),
            compression_level: Default::default(),
            compress_image_to_jpeg: Default::default(),
            header_mode: HeaderMode::Complete,
        }
    }
}

impl TryFrom<DirsOptions> for Builder {
    type Error = api_core::Error;
    fn try_from(value: DirsOptions) -> Result<Self, Self::Error> {
        Ok(Self {
            contents: (&value).try_into()?,
            initial_dir_options: value,
            ..Default::default()
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveMangaError {
    #[error("This manga is not found in the package content")]
    NotFound(Uuid),
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveCoverError {
    #[error("This cover is not found in the package content")]
    NotFound(Uuid),
    #[error("Cannot have an empty cover array")]
    CannotBeSingle,
    #[error("Both of the covers arrays and chapters map is nearly empty. Please remove the manga instead")]
    RemoveMangaInstead,
}

#[derive(Debug, thiserror::Error)]
pub enum RemoveChapterError {
    #[error("This chapter is not found in the package content")]
    NotFound(Uuid),
    #[error("Both of the covers arrays and chapters map is nearly empty. Please remove the manga instead")]
    RemoveMangaInstead,
    #[error("the `data` and the `data-saver` chapter images array is empty. Please remove the chapter instead")]
    RemoveChapterInstead,
}

impl Builder {
    pub fn new(dirs: DirsOptions) -> Self {
        Self {
            initial_dir_options: dirs,
            ..Default::default()
        }
    }
    pub fn set_compress_image_to_jpeg(mut self, compress_image_to_jpeg: bool) -> Self {
        self.compress_image_to_jpeg = compress_image_to_jpeg;
        self
    }
    pub fn set_tar_header_mode(mut self, mode: HeaderMode) -> Self {
        self.header_mode = mode;
        self
    }
    pub fn set_content(mut self, content: PackageContents) -> Self {
        self.contents = content;
        self
    }
    pub fn add_manga(&mut self, id: Uuid) -> ThisResult<()> {
        let manga_data: MangaObject = self.initial_dir_options.pull(id)?;
        let cover_id = {
            let manga_data_cover_id = manga_data
                .find_first_relationships(RelationshipType::CoverArt)
                .ok_or(api_core::Error::MissingRelationships(vec![
                    RelationshipType::CoverArt,
                ]))?
                .id;
            if let Some(data) = self
                .initial_dir_options
                .pull_all_covers()
                .ok()
                .and_then(|pull| pull.flatten().find(|data| data.id == manga_data_cover_id))
            {
                data.id
            } else {
                self.initial_dir_options
                    .pull_all_covers()?
                    .flatten()
                    .to_filtered(CoverListDataPullFilterParams {
                        manga_ids: vec![manga_data.id],
                        ..Default::default()
                    })
                    .map(|e| e.id)
                    .collect::<Vec<_>>()
                    .random()
                    .ok_or(api_core::Error::MissingRelationships(vec![
                        RelationshipType::CoverArt,
                    ]))?
            }
        };
        self.contents
            .data
            .entry(manga_data.id)
            .or_default()
            .covers
            .push(cover_id);
        Ok(())
    }
    pub fn add_cover(&mut self, id: Uuid) -> ThisResult<()> {
        let cover_data: CoverObject = self.initial_dir_options.pull(id)?;
        let manga_id = {
            let cover_data_manga_id = cover_data
                .find_first_relationships(RelationshipType::Manga)
                .ok_or(api_core::Error::MissingRelationships(vec![
                    RelationshipType::Manga,
                ]))?
                .id;
            ({
                let d: MangaObject = self.initial_dir_options.pull(cover_data_manga_id)?;
                d
            })
            .id
        };
        let covers = &mut self.contents.data.entry(manga_id).or_default().covers;
        if !covers.contains(&cover_data.id) {
            covers.push(cover_data.id)
        }
        Ok(())
    }
    pub fn add_chapter(&mut self, id: Uuid, mode: ChapterImagesMode) -> ThisResult<()> {
        let chapter_data: ChapterObject = self.initial_dir_options.pull(id)?;
        let images: ChapterImagesData = self.initial_dir_options.pull(id)?;
        let manga = {
            let chapter_data_manga_id = chapter_data
                .find_first_relationships(RelationshipType::Manga)
                .ok_or(api_core::Error::MissingRelationships(vec![
                    RelationshipType::Manga,
                ]))?
                .id;
            {
                let d: MangaObject = self.initial_dir_options.pull(chapter_data_manga_id)?;
                d
            }
        };
        let chapter_images = self
            .contents
            .data
            .entry(manga.id)
            .or_insert(PMangaObject {
                covers: vec![
                    manga
                        .find_first_relationships(RelationshipType::CoverArt)
                        .ok_or(api_core::Error::MissingRelationships(vec![
                            RelationshipType::CoverArt,
                        ]))?
                        .id,
                ],
                ..Default::default()
            })
            .chapters
            .entry(chapter_data.id)
            .or_default();
        match mode {
            ChapterImagesMode::Data => chapter_images.data = images.data,
            ChapterImagesMode::DataSaver => chapter_images.data_saver = images.data_saver,
        };
        Ok(())
    }
    pub fn remove_manga(&mut self, id: Uuid) -> Result<(), RemoveMangaError> {
        if self.contents.data.remove(&id).is_none() {
            Err(RemoveMangaError::NotFound(id))
        } else {
            Ok(())
        }
    }
    pub fn remove_cover(&mut self, id: Uuid) -> Result<(), RemoveCoverError> {
        if let Some((_, manga_data, index)) =
            self.contents
                .data
                .iter_mut()
                .find_map(|(manga_id, manga_data)| {
                    manga_data
                        .covers
                        .iter()
                        .enumerate()
                        .find_map(
                            |(index, cover_id)| if *cover_id == id { Some(index) } else { None },
                        )
                        .map(|index| (manga_id, manga_data, index))
                })
        {
            let covers = &mut manga_data.covers;
            if covers.len() > 1 {
                covers.remove(index);
                Ok(())
            } else if manga_data.chapters.len() <= 1 {
                Err(RemoveCoverError::RemoveMangaInstead)
            } else {
                Err(RemoveCoverError::CannotBeSingle)
            }
        } else {
            Err(RemoveCoverError::NotFound(id))
        }
    }
    pub fn remove_chapter(&mut self, id: Uuid) -> Result<(), RemoveChapterError> {
        if let Some(manga_id) = self
            .contents
            .data
            .iter()
            .find_map(|(manga_id, manga_data)| {
                if manga_data.chapters.keys().any(|_id| *_id == id) {
                    Some(*manga_id)
                } else {
                    None
                }
            })
        {
            let manga_data = self
                .contents
                .data
                .get_mut(&manga_id)
                .ok_or(RemoveChapterError::NotFound(id))?;
            if manga_data.chapters.len() <= 1 && manga_data.covers.len() <= 1 {
                Err(RemoveChapterError::RemoveMangaInstead)
            } else {
                manga_data
                    .chapters
                    .remove(&id)
                    .map(|_| {})
                    .ok_or(RemoveChapterError::NotFound(id))
            }
        } else {
            Err(RemoveChapterError::NotFound(id))
        }
    }
    pub fn clear_chapter_images(
        &mut self,
        id: Uuid,
        mode: ChapterImagesMode,
    ) -> Result<(), RemoveChapterError> {
        if let Some(manga_id) = self
            .contents
            .data
            .iter()
            .find_map(|(manga_id, manga_data)| {
                if manga_data.chapters.keys().any(|_id| *_id == id) {
                    Some(*manga_id)
                } else {
                    None
                }
            })
        {
            let manga_data = self
                .contents
                .data
                .get_mut(&manga_id)
                .ok_or(RemoveChapterError::NotFound(id))?;
            let should_remove_manga =
                manga_data.deref().chapters.len() <= 1 && manga_data.covers.len() <= 1;
            let chapters_data = manga_data
                .chapters
                .get_mut(&id)
                .ok_or(RemoveChapterError::NotFound(id))?;
            let should_remove_chapter =
                chapters_data.data.is_empty() || chapters_data.data_saver.is_empty();
            if should_remove_manga {
                Err(RemoveChapterError::RemoveMangaInstead)
            } else if should_remove_chapter {
                Err(RemoveChapterError::RemoveChapterInstead)
            } else {
                match mode {
                    ChapterImagesMode::Data => chapters_data.data.clear(),
                    ChapterImagesMode::DataSaver => chapters_data.data_saver.clear(),
                };
                Ok(())
            }
        } else {
            Err(RemoveChapterError::NotFound(id))
        }
    }
    pub fn zstd_compressed_images(&mut self, compressed_images: bool) {
        self.contents
            .options
            .get_or_insert_with(Default::default)
            .zstd_compressed_images = compressed_images;
    }
    pub fn set_compression_level(&mut self, compression_level: i32) {
        self.compression_level = compression_level;
    }
    pub fn get_compression_level(&self) -> i32 {
        self.compression_level
    }
    pub fn zstd_compressed_metadata(&mut self, compressed_metadata: bool) {
        self.contents
            .options
            .get_or_insert_with(Default::default)
            .zstd_compressed_metadata = compressed_metadata;
    }
    pub fn build<W: Write>(self, writer: W) -> ThisResult<PackageContents> {
        BuilderInner::new(self, writer)?.build()
    }
    fn get_to_use_paths(&self) -> Vec<PathBuf> {
        let mut paths = Vec::new();
        for (manga_id, manga_data) in &self.contents.data {
            paths.push(
                self.initial_dir_options
                    .mangas_add(format!("{manga_id}.json")),
            );
            for cover_id in &manga_data.covers {
                paths.push(
                    self.initial_dir_options
                        .covers_add(format!("{cover_id}.json")),
                );
                if let Ok(cover_data) =
                    Pull::<CoverObject, Uuid>::pull(&self.initial_dir_options, *cover_id)
                {
                    paths.push(
                        self.initial_dir_options
                            .cover_images_add(cover_data.attributes.file_name),
                    );
                }
            }
            for (chapter_id, chapter_data) in &manga_data.chapters {
                paths.push(
                    self.initial_dir_options
                        .chapters_id_add(*chapter_id)
                        .join("data.json"),
                );
                for filename in &chapter_data.data {
                    paths.push(
                        self.initial_dir_options
                            .chapters_id_data_add(*chapter_id)
                            .join(filename),
                    );
                }
                for filename in &chapter_data.data_saver {
                    paths.push(
                        self.initial_dir_options
                            .chapters_id_data_saver_add(*chapter_id)
                            .join(filename),
                    );
                }
            }
        }
        paths
    }
    pub fn create_dict(&self, max_size: usize) -> io::Result<Vec<u8>> {
        zstd::dict::from_files(self.get_to_use_paths(), max_size)
    }
    pub fn get_package_contents(&self) -> &PackageContents {
        &self.contents
    }
}
