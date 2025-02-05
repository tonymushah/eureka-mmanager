use mangadex_api_input_types::manga::list::MangaListParams;
use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::{
    ContentRating, Demographic, Language, MangaDexDateTime, MangaStatus, RelationshipType,
    TagSearchMode,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{data_pulls::Validate, option_bool_match};

#[derive(Debug, Clone, Default, Deserialize, Serialize)]
pub struct MangaListDataPullFilterParams {
    pub title: Option<String>,
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
            title: value.title,
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
        if !self.authors.is_empty() {
            let authors_ids = input
                .find_relationships(RelationshipType::Author)
                .iter()
                .map(|a| a.id)
                .collect::<Vec<_>>();
            Some(
                authors_ids
                    .iter()
                    .filter(|id| self.authors.contains(*id))
                    .all(|a| self.authors.contains(a)),
            )
        } else {
            None
        }
    }
    fn validate_artists(&self, input: &MangaObject) -> Option<bool> {
        if !self.artists.is_empty() {
            let artists_ids = input
                .find_relationships(RelationshipType::Artist)
                .iter()
                .map(|a| a.id)
                .collect::<Vec<_>>();
            Some(
                artists_ids
                    .iter()
                    .filter(|a| self.artists.contains(*a))
                    .all(|a| self.artists.contains(a)),
            )
        } else {
            None
        }
    }
    fn validate_year(&self, input: &MangaObject) -> Option<bool> {
        let year = self.year?;
        let input_year = input.attributes.year?;
        Some(year == input_year)
    }
    fn validate_included_tags(&self, input: &MangaObject) -> Option<bool> {
        let mode = self.included_tags_mode.unwrap_or(TagSearchMode::And);
        if !self.included_tags.is_empty() {
            let input_tags = input
                .attributes
                .tags
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>();
            let res = match mode {
                TagSearchMode::And => input_tags
                    .iter()
                    .filter(|t| self.included_tags.contains(t))
                    .all(|t| self.included_tags.contains(t)),
                TagSearchMode::Or => self.included_tags.iter().any(|t| input_tags.contains(t)),
            };
            Some(res)
        } else {
            None
        }
    }
    fn validate_excluded_tags(&self, input: &MangaObject) -> Option<bool> {
        let mode = self.excluded_tags_mode.unwrap_or(TagSearchMode::Or);
        if !self.excluded_tags.is_empty() {
            let input_tags = input
                .attributes
                .tags
                .iter()
                .map(|t| t.id)
                .collect::<Vec<_>>();
            let res = match mode {
                TagSearchMode::And => input_tags
                    .iter()
                    .filter(|t| self.excluded_tags.contains(t))
                    .all(|t| self.excluded_tags.contains(t)),
                TagSearchMode::Or => self.excluded_tags.iter().any(|t| input_tags.contains(t)),
            };
            Some(!res)
        } else {
            None
        }
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
            Some(self.publication_demographic.contains(option_bool_match!(
                input.attributes.publication_demographic.as_ref()
            )))
        } else {
            None
        }
    }
    fn validate_content_rating(&self, input: &MangaObject) -> Option<bool> {
        if !self.content_rating.is_empty() {
            Some(
                self.content_rating
                    .contains(input.attributes.content_rating.as_ref()?),
            )
        } else {
            None
        }
    }
    fn validate_created_at_since(&self, input: &MangaObject) -> Option<bool> {
        Some(self.created_at_since?.as_ref() < input.attributes.created_at.as_ref())
    }
    fn validate_updated_at_since(&self, input: &MangaObject) -> Option<bool> {
        Some(
            self.updated_at_since?.as_ref()
                < input.attributes.updated_at.as_ref().map(|d| d.as_ref())?,
        )
    }
    fn validate_title(&self, input: &MangaObject) -> Option<bool> {
        let title = self.title.as_ref()?;
        let title_regex = regex::Regex::new(title).ok()?;
        Some(
            input
                .attributes
                .title
                .values()
                .any(|title_item| title_regex.is_match(title_item))
                || input.attributes.alt_titles.iter().any(|item| {
                    item.values()
                        .any(|title_item| title_regex.is_match(title_item))
                }),
        )
    }
}

impl Validate<MangaObject> for MangaListDataPullFilterParams {
    fn is_valid(&self, input: &MangaObject) -> bool {
        let validations = vec![
            self.validate_title(input),
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
