use mangadex_api_input_types::{
    chapter::list::ChapterListParams, manga::aggregate::MangaAggregateParam,
};
use mangadex_api_schema_rust::{
    v5::{ChapterObject, MangaAttributes},
    ApiObjectNoRelationships,
};
use mangadex_api_types_rust::{ContentRating, Language, MangaDexDateTime, RelationshipType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{data_pulls::Validate, option_bool_match};

/// This parameter that allows you to filter a [`tokio_stream::Stream<Item = ChapterObject>`] or a [`Iterator<Item = ChapterObject>`]
/// via [`crate::prelude::IntoParamedFilteredStream`] for an async stream
/// or [`crate::prelude::IntoFiltered`] for an non-blocking iterator
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct ChapterListDataPullFilterParams {
    pub title: Option<String>,
    pub groups: Vec<Uuid>,
    pub uploaders: Vec<Uuid>,
    pub volumes: Vec<String>,
    pub manga_ids: Vec<Uuid>,
    /// Chapter number in the series or volume.
    pub chapters: Vec<String>,
    pub translated_languages: Vec<Language>,
    pub original_languages: Vec<Language>,
    pub excluded_original_languages: Vec<Language>,
    pub content_rating: Vec<ContentRating>,
    /// Groups to exclude from the results.
    pub excluded_groups: Vec<Uuid>,
    /// Uploaders to exclude from the results.
    pub excluded_uploaders: Vec<Uuid>,
    pub created_at_since: Option<MangaDexDateTime>,
    /// DateTime string with following format: `YYYY-MM-DDTHH:MM:SS`.
    pub updated_at_since: Option<MangaDexDateTime>,
    /// DateTime string with following format: `YYYY-MM-DDTHH:MM:SS`.
    pub publish_at_since: Option<MangaDexDateTime>,
}

impl From<ChapterListParams> for ChapterListDataPullFilterParams {
    fn from(value: ChapterListParams) -> Self {
        Self {
            title: value.title,
            groups: value.groups,
            uploaders: value.uploaders,
            volumes: value.volumes,
            chapters: value.chapters,
            translated_languages: value.translated_languages,
            original_languages: value.original_languages,
            excluded_original_languages: value.excluded_original_languages,
            content_rating: value.content_rating,
            excluded_groups: value.excluded_groups,
            excluded_uploaders: value.excluded_uploaders,
            created_at_since: value.created_at_since,
            updated_at_since: value.updated_at_since,
            publish_at_since: value.publish_at_since,
            manga_ids: if let Some(manga_id) = value.manga_id {
                vec![manga_id]
            } else {
                Default::default()
            },
        }
    }
}

impl From<MangaAggregateParam> for ChapterListDataPullFilterParams {
    fn from(value: MangaAggregateParam) -> Self {
        Self {
            manga_ids: vec![value.manga_id],
            translated_languages: value.translated_language,
            groups: value.groups,
            ..Default::default()
        }
    }
}

impl ChapterListDataPullFilterParams {
    fn validate_title(&self, input: &ChapterObject) -> Option<bool> {
        let title = self.title.as_ref()?;
        let title_regex = regex::Regex::new(title).ok()?;
        let input_title = option_bool_match!(input.attributes.title.as_ref());
        Some(title_regex.is_match(input_title))
    }
    fn validate_groups(&self, input: &ChapterObject) -> Option<bool> {
        let groups = &self.groups;
        if !groups.is_empty() {
            let input_groups = input
                .find_relationships(RelationshipType::ScanlationGroup)
                .into_iter()
                .map(|o| o.id)
                .collect::<Vec<_>>();
            Some(
                input_groups
                    .iter()
                    .filter(|id| groups.contains(*id))
                    .all(|id| groups.contains(id)),
            )
        } else {
            None
        }
    }
    fn validate_uploaders(&self, input: &ChapterObject) -> Option<bool> {
        let uploaders = &self.uploaders;

        if !uploaders.is_empty() {
            let input_uploaders: Vec<Uuid> = {
                let mut us = input
                    .find_relationships(RelationshipType::User)
                    .into_iter()
                    .map(|o| o.id)
                    .collect::<Vec<_>>();
                if let Some(u) = input.attributes.uploader.as_ref() {
                    us.push(*u);
                }
                us
            };
            Some(uploaders.iter().all(|g| input_uploaders.contains(g)))
        } else {
            None
        }
    }
    fn validate_volumes(&self, input: &ChapterObject) -> Option<bool> {
        let volumes = &self.volumes;

        if !volumes.is_empty() {
            let none = String::from("none");
            let input_volumes = input.attributes.volume.as_ref().unwrap_or(&none);
            Some(volumes.contains(input_volumes))
        } else {
            None
        }
    }
    fn validate_chapters(&self, input: &ChapterObject) -> Option<bool> {
        let chapters = &self.chapters;
        if !chapters.is_empty() {
            let none = String::from("none");
            let input_chapter = input.attributes.chapter.as_ref().unwrap_or(&none);
            Some(chapters.contains(input_chapter))
        } else {
            None
        }
    }
    fn validate_translated_languages(&self, input: &ChapterObject) -> Option<bool> {
        let tl = &self.translated_languages;
        let input_tl = &input.attributes.translated_language;
        if !tl.is_empty() {
            Some(tl.contains(input_tl))
        } else {
            None
        }
    }
    fn validate_original_language(&self, input: &ChapterObject) -> Option<bool> {
        let tl = &self.original_languages;

        if !tl.is_empty() {
            let input_tl = {
                let manga: ApiObjectNoRelationships<MangaAttributes> = option_bool_match!(input
                    .find_first_relationships(RelationshipType::Manga)?
                    .clone()
                    .try_into()
                    .ok());
                manga.attributes.original_language
            };
            Some(tl.contains(&input_tl))
        } else {
            None
        }
    }
    fn validate_excluded_original_language(&self, input: &ChapterObject) -> Option<bool> {
        let tl = &self.excluded_original_languages;

        if !tl.is_empty() {
            let input_tl = {
                let manga: ApiObjectNoRelationships<MangaAttributes> = option_bool_match!(input
                    .find_first_relationships(RelationshipType::Manga)?
                    .clone()
                    .try_into()
                    .ok());
                manga.attributes.original_language
            };
            Some(!tl.contains(&input_tl))
        } else {
            None
        }
    }
    fn validate_content_rating(&self, input: &ChapterObject) -> Option<bool> {
        let tl = &self.content_rating;

        if !tl.is_empty() {
            let input_tl = {
                let manga: ApiObjectNoRelationships<MangaAttributes> = option_bool_match!(input
                    .find_first_relationships(RelationshipType::Manga)?
                    .clone()
                    .try_into()
                    .ok());
                option_bool_match!(manga.attributes.content_rating)
            };
            Some(tl.contains(&input_tl))
        } else {
            None
        }
    }
    fn validate_excluded_groups(&self, input: &ChapterObject) -> Option<bool> {
        let groups = &self.excluded_groups;

        if !groups.is_empty() {
            let input_groups = input
                .find_relationships(RelationshipType::ScanlationGroup)
                .into_iter()
                .map(|o| o.id)
                .collect::<Vec<_>>();
            Some(!groups.iter().any(|g| input_groups.contains(g)))
        } else {
            None
        }
    }
    fn validate_excluded_uploaders(&self, input: &ChapterObject) -> Option<bool> {
        let uploaders = &self.excluded_groups;

        if !uploaders.is_empty() {
            let input_uploaders: Vec<Uuid> = {
                let mut us = input
                    .find_relationships(RelationshipType::User)
                    .into_iter()
                    .map(|o| o.id)
                    .collect::<Vec<_>>();
                if let Some(u) = input.attributes.uploader.as_ref() {
                    us.push(*u);
                }
                us
            };
            Some(!uploaders.iter().all(|g| input_uploaders.contains(g)))
        } else {
            None
        }
    }
    fn validate_created_at_since(&self, input: &ChapterObject) -> Option<bool> {
        Some(self.created_at_since?.as_ref() < input.attributes.created_at.as_ref())
    }
    fn validate_updated_at_since(&self, input: &ChapterObject) -> Option<bool> {
        Some(self.updated_at_since?.as_ref() < input.attributes.updated_at?.as_ref())
    }
    fn validate_publish_at_since(&self, input: &ChapterObject) -> Option<bool> {
        Some(self.publish_at_since?.as_ref() < input.attributes.publish_at?.as_ref())
    }
    fn validate_manga_id(&self, input: &ChapterObject) -> Option<bool> {
        let tl = &self.manga_ids;
        if tl.is_empty() {
            return None;
        }
        let input_tl = {
            let manga = option_bool_match!(input.find_first_relationships(RelationshipType::Manga));
            manga.id
        };
        Some(tl.contains(&input_tl))
    }
}

impl Validate<ChapterObject> for ChapterListDataPullFilterParams {
    fn is_valid(&self, input: &ChapterObject) -> bool {
        let start_val = [
            self.validate_title(input),
            self.validate_groups(input),
            self.validate_uploaders(input),
            self.validate_volumes(input),
            self.validate_chapters(input),
            self.validate_translated_languages(input),
            self.validate_original_language(input),
            self.validate_excluded_original_language(input),
            self.validate_content_rating(input),
            self.validate_excluded_groups(input),
            self.validate_excluded_uploaders(input),
            self.validate_created_at_since(input),
            self.validate_updated_at_since(input),
            self.validate_publish_at_since(input),
            self.validate_manga_id(input),
        ];
        let validations = start_val.iter().flatten().copied().collect::<Vec<bool>>();
        let mut is_valid = true;
        for validation in validations {
            is_valid = is_valid && validation
        }
        is_valid
    }
}
