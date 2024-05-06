use mangadex_api_input_types::chapter::list::ChapterListParams;
use mangadex_api_types_rust::{ContentRating, Language, MangaDexDateTime};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ChapterListDataPullFilterParams {
    pub title: Option<String>,
    pub groups: Vec<Uuid>,
    pub uploaders: Vec<Uuid>,
    pub volumes: Vec<String>,
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
        }
    }
}
