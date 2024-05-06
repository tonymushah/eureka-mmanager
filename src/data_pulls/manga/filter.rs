use mangadex_api_input_types::manga::list::MangaListParams;
use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::{
    ContentRating, Demographic, Language, MangaDexDateTime, MangaStatus, RelationshipType,
    TagSearchMode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::data_pulls::Validate;

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MangaListDataPullFilterParams {
    pub author_or_artist: Option<Uuid>,
    pub authors: Vec<Uuid>,
    pub artists: Vec<Uuid>,
    pub year: Option<u16>,
    pub included_tags: Vec<Uuid>,
    pub included_tags_mode: Option<TagSearchMode>,
    pub excluded_tags: Vec<Uuid>,
    pub excluded_tags_mode: Option<TagSearchMode>,
    pub status: Vec<MangaStatus>,
    pub original_language: Vec<Language>,
    pub excluded_original_language: Vec<Language>,
    pub publication_demographic: Vec<Demographic>,
    pub content_rating: Vec<ContentRating>,
    pub created_at_since: Option<MangaDexDateTime>,
    pub updated_at_since: Option<MangaDexDateTime>,
    pub group: Option<Uuid>,
}

impl From<MangaListParams> for MangaListDataPullFilterParams {
    fn from(value: MangaListParams) -> Self {
        Self {
            author_or_artist: value.author_or_artist,
            authors: value.authors,
            artists: value.artists,
            year: value.year,
            included_tags: value.included_tags,
            included_tags_mode: value.included_tags_mode,
            excluded_tags: value.excluded_tags,
            excluded_tags_mode: value.excluded_tags_mode,
            status: value.status,
            original_language: value.original_language,
            excluded_original_language: value.excluded_original_language,
            publication_demographic: value.publication_demographic,
            content_rating: value.content_rating,
            created_at_since: value.created_at_since,
            updated_at_since: value.updated_at_since,
            group: value.group,
        }
    }
}

impl MangaListDataPullFilterParams {
    fn validate_author_or_artist(&self, input: &MangaObject) -> Option<bool> {
        if let Some(id) = self.author_or_artist {
            if !input
                .find_relationships(RelationshipType::Author)
                .iter()
                .any(|a| a.id == id)
            {
                Some(
                    input
                        .find_relationships(RelationshipType::Artist)
                        .iter()
                        .any(|a| a.id == id),
                )
            } else {
                Some(true)
            }
        } else {
            None
        }
    }
    fn validate_authors(&self, input: &MangaObject) -> Option<bool> {
        let authors_ids = input
            .find_relationships(RelationshipType::Author)
            .iter()
            .map(|a| a.id)
            .collect::<Vec<_>>();
        if !self.authors.is_empty() && !authors_ids.is_empty() {
            Some(self.authors.iter().all(|a| authors_ids.contains(a)))
        } else {
            None
        }
    }
    fn validate_artists(&self, input: &MangaObject) -> Option<bool> {
        let artists_ids = input
            .find_relationships(RelationshipType::Artist)
            .iter()
            .map(|a| a.id)
            .collect::<Vec<_>>();
        if !self.artists.is_empty() && !artists_ids.is_empty() {
            Some(self.artists.iter().all(|a| artists_ids.contains(a)))
        } else {
            None
        }
    }
    fn validate_year(&self, input: &MangaObject) -> Option<bool> {
        let input_year = input.attributes.year?;
        Some(self.year? == input_year)
    }
    fn validate_included_tags(&self, input: &MangaObject) -> Option<bool> {
        let mode = self.included_tags_mode.unwrap_or(TagSearchMode::And);
        let input_tags = input
            .attributes
            .tags
            .iter()
            .map(|t| t.id)
            .collect::<Vec<_>>();
        let res = match mode {
            TagSearchMode::And => self.included_tags.iter().all(|t| input_tags.contains(t)),
            TagSearchMode::Or => self.included_tags.iter().any(|t| input_tags.contains(t)),
        };
        Some(res)
    }
    fn validate_excluded_tags(&self, input: &MangaObject) -> Option<bool> {
        let mode = self.excluded_tags_mode.unwrap_or(TagSearchMode::Or);
        let input_tags = input
            .attributes
            .tags
            .iter()
            .map(|t| t.id)
            .collect::<Vec<_>>();
        let res = match mode {
            TagSearchMode::And => self.excluded_tags.iter().all(|t| input_tags.contains(t)),
            TagSearchMode::Or => self.excluded_tags.iter().any(|t| input_tags.contains(t)),
        };
        Some(res)
    }
    fn validate_status(&self, input: &MangaObject) -> Option<bool> {
        if !self.status.is_empty() {
            Some(self.status.contains(&input.attributes.status))
        } else {
            None
        }
    }
    fn validate_original_language(&self, input: &MangaObject) -> Option<bool> {
        if !self.original_language.is_empty() {
            Some(
                self.original_language
                    .contains(&input.attributes.original_language),
            )
        } else {
            None
        }
    }
    fn validate_excluded_original_language(&self, input: &MangaObject) -> Option<bool> {
        if !self.excluded_original_language.is_empty() {
            Some(
                self.excluded_original_language
                    .contains(&input.attributes.original_language),
            )
        } else {
            None
        }
    }
    fn validate_publication_demographic(&self, input: &MangaObject) -> Option<bool> {
        if !self.publication_demographic.is_empty() {
            Some(
                self.publication_demographic
                    .contains(&input.attributes.publication_demographic?),
            )
        } else {
            None
        }
    }
    fn validate_content_rating(&self, input: &MangaObject) -> Option<bool> {
        if !self.content_rating.is_empty() {
            Some(
                self.content_rating
                    .contains(&input.attributes.content_rating?),
            )
        } else {
            None
        }
    }
    fn validate_created_at_since(&self, input: &MangaObject) -> Option<bool> {
        Some(self.created_at_since?.as_ref() < input.attributes.created_at.as_ref())
    }
    fn validate_updated_at_since(&self, input: &MangaObject) -> Option<bool> {
        Some(self.updated_at_since?.as_ref() < input.attributes.updated_at?.as_ref())
    }
}

impl Validate<MangaObject> for MangaListDataPullFilterParams {
    fn is_valid(&self, input: &MangaObject) -> bool {
        let validations = vec![
            self.validate_artists(input),
            self.validate_author_or_artist(input),
            self.validate_authors(input),
            self.validate_content_rating(input),
            self.validate_created_at_since(input),
            self.validate_excluded_original_language(input),
            self.validate_excluded_tags(input),
            self.validate_included_tags(input),
            self.validate_original_language(input),
            self.validate_publication_demographic(input),
            self.validate_status(input),
            self.validate_updated_at_since(input),
            self.validate_year(input),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<bool>>();
        let mut is_valid = true;
        for validation in validations {
            is_valid = is_valid && validation
        }
        is_valid
    }
}
