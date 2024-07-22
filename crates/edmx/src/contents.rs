use std::{
    collections::HashMap,
    ops::{Add, AddAssign},
};

use api_core::{
    data_pulls::{chapter::images::ChapterImagesData, Pull},
    DirsOptions,
};
use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::RelationshipType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PChapterObject {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub data: Vec<String>,
    #[serde(default, rename = "data-saver", skip_serializing_if = "Vec::is_empty")]
    pub data_saver: Vec<String>,
}

impl From<PChapterObject> for ChapterImagesData {
    fn from(value: PChapterObject) -> Self {
        Self {
            data: value.data,
            data_saver: value.data_saver,
        }
    }
}

impl From<ChapterImagesData> for PChapterObject {
    fn from(value: ChapterImagesData) -> Self {
        Self {
            data: value.data,
            data_saver: value.data_saver,
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PMangaObject {
    pub covers: Vec<Uuid>,
    pub chapters: HashMap<Uuid, PChapterObject>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct PackageContents {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub options: Option<DirsOptions>,
    pub data: HashMap<Uuid, PMangaObject>,
}

impl TryFrom<&DirsOptions> for PackageContents {
    type Error = api_core::Error;
    fn try_from(value: &DirsOptions) -> Result<Self, Self::Error> {
        let mut _covers = value.pull_all_covers()?.flatten().fold(
            HashMap::<Uuid, Vec<Uuid>>::new(),
            |mut acc, current| {
                if let Some(manga) = current.find_first_relationships(RelationshipType::Manga) {
                    acc.entry(manga.id).or_default().push(current.id);
                }
                acc
            },
        );
        let mut _chapters = value
            .pull_all_chapter()?
            .flatten()
            .flat_map(
                |chapter| -> Result<(ChapterObject, ChapterImagesData), api_core::Error> {
                    value.pull(chapter.id).map(|data| (chapter, data))
                },
            )
            .fold(
                HashMap::<Uuid, HashMap<Uuid, PChapterObject>>::new(),
                |mut acc, (chapter, images)| {
                    if let Some(manga) = chapter.find_first_relationships(RelationshipType::Manga) {
                        acc.entry(manga.id)
                            .or_default()
                            .insert(chapter.id, images.into());
                    }
                    acc
                },
            );
        let data = value.pull_all_mangas()?.flatten().fold(
            HashMap::<Uuid, PMangaObject>::new(),
            |mut acc, manga| {
                if let Some(covers) = _covers.remove(&manga.id) {
                    if let Some(chapters) = _chapters.remove(&manga.id) {
                        acc.insert(manga.id, PMangaObject { covers, chapters });
                    }
                }
                acc
            },
        );
        Ok(Self {
            options: None,
            data,
        })
    }
}

impl Add for PackageContents {
    type Output = PackageContents;
    fn add(mut self, rhs: Self) -> Self::Output {
        for (manga_id, mut manga_data) in rhs.data {
            let insert_manga_data_here = self.data.entry(manga_id).or_default();
            insert_manga_data_here.covers.append(&mut manga_data.covers);
            insert_manga_data_here.covers.dedup();
            for (chapter_id, chapter_data) in manga_data.chapters {
                insert_manga_data_here
                    .chapters
                    .insert(chapter_id, chapter_data);
            }
        }
        self
    }
}

impl AddAssign for PackageContents {
    fn add_assign(&mut self, rhs: Self) {
        *self = self.clone() + rhs
    }
}
