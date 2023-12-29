use mangadex_api_schema_rust::v5::manga_aggregate::{ChapterAggregate, VolumeAggregate};
use mangadex_api_types_rust::Language;
use serde::Deserialize;
use uuid::Uuid;

use super::chapter::GetAllChapter;

pub mod stream;

pub trait ShouldBe<'a, T: ?Sized> {
    type Output;
    fn should_be(&'a mut self, input: T) -> Self::Output;
}

pub enum FromWhere<'a> {
    Volume(&'a mut VolumeAggregate),
    Chapter(&'a mut ChapterAggregate),
}

pub enum IsHere {
    AlreadyHere,
    Inserted,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(default)]
pub struct MangaAggregateParams {
    pub translated_language: Vec<Language>,
    pub groups: Vec<Uuid>,
    #[serde(flatten)]
    pub additional_params: GetAllChapter,
}
