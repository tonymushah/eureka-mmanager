pub mod chapter_ids;
pub mod chapters;
pub mod content_rating;
pub mod created_at_since;
pub mod excluded_groups;
pub mod excluded_original_languages;
pub mod excluded_uploaders;
pub mod groups;
pub mod include_empty_pages;
pub mod includes;
pub mod manga_id;
pub mod original_languages;
pub mod publish_at_since;
pub mod title;
pub mod translated_languages;
pub mod updated_at_since;
pub mod uploaders;
pub mod volumes;

use mangadex_api_input_types::chapter::list::ChapterListParams;
use mangadex_api_schema_rust::v5::ChapterObject;

use crate::utils::chapter::filter::{
    chapter_ids::filter_fn_via_chapter_ids, chapters::filter_fn_via_chapters,
    content_rating::filter_fn_via_content_rating, created_at_since::filter_fn_via_created_at_since,
    groups::filter_fn_via_groups, include_empty_pages::filter_fn_via_include_empty_pages,
    manga_id::filter_fn_via_manga_id, original_languages::filter_fn_via_original_languages,
    publish_at_since::filter_fn_via_publish_at_since, title::filter_fn_via_title,
    translated_languages::filter_fn_via_translated_languages,
    updated_at_since::filter_fn_via_updated_at_since, uploaders::filter_fn_via_uploaders,
    volumes::filter_fn_via_volumes,
};

use self::{
    excluded_groups::filter_fn_via_excluded_groups,
    excluded_original_languages::filter_fn_via_excluded_original_languages,
    excluded_uploaders::filter_fn_via_excluded_uploaders,
};

pub fn filter_fn_excluded<'a>(item: &'a ChapterObject, param: &'a ChapterListParams) -> bool {
    let excluded_groups_filter = {
        let excluded_groups = &param.excluded_groups;
        if !excluded_groups.is_empty() {
            filter_fn_via_excluded_groups(item, excluded_groups)
        } else {
            true
        }
    };
    let excluded_original_languages_filter = {
        let excluded_original_languages = &param.excluded_original_languages;
        if !excluded_original_languages.is_empty() {
            filter_fn_via_excluded_original_languages(item, excluded_original_languages)
        } else {
            true
        }
    };
    let excluded_uploaders_filter = {
        let excluded_uploaders = &param.excluded_uploaders;
        if !excluded_uploaders.is_empty() {
            filter_fn_via_excluded_uploaders(item, excluded_uploaders)
        } else {
            true
        }
    };
    excluded_groups_filter && excluded_original_languages_filter && excluded_uploaders_filter
}

pub fn filter<'a>(item: &'a ChapterObject, param: &'a ChapterListParams) -> bool {
    let chapter_ids_filters = {
        let chapter_ids = &param.chapter_ids;
        if !chapter_ids.is_empty() {
            filter_fn_via_chapter_ids(item, chapter_ids)
        } else {
            true
        }
    };
    let chapters_filters = {
        let chapters = &param.chapters;
        if !chapters.is_empty() {
            filter_fn_via_chapters(item, chapters)
        } else {
            true
        }
    };
    let content_rating_filters = {
        let content_rating = &param.content_rating;
        if !content_rating.is_empty() {
            filter_fn_via_content_rating(item, content_rating)
        } else {
            true
        }
    };
    let created_at_since_filters = {
        if let Some(ref created_at_since) = param.created_at_since {
            filter_fn_via_created_at_since(item, created_at_since)
        } else {
            true
        }
    };
    let groups_filters = {
        let groups = &param.groups;
        if !groups.is_empty() {
            filter_fn_via_groups(item, groups)
        } else {
            true
        }
    };
    let include_empty_pages_filters = {
        if let Some(ref include_empty_pages) = param.include_empty_pages {
            filter_fn_via_include_empty_pages(item, include_empty_pages)
        } else {
            true
        }
    };
    let manga_id_filters = {
        if let Some(ref manga_id) = param.manga_id {
            filter_fn_via_manga_id(item, manga_id)
        } else {
            true
        }
    };
    let original_languages_filters = {
        let original_languages = &param.original_languages;
        if !original_languages.is_empty() {
            filter_fn_via_original_languages(item, original_languages)
        } else {
            true
        }
    };
    let publish_at_since_filters = {
        if let Some(ref publish_at_since) = param.publish_at_since {
            filter_fn_via_publish_at_since(item, publish_at_since)
        } else {
            true
        }
    };
    let title_filters = {
        if let Some(ref title) = param.title {
            filter_fn_via_title(item, title)
        } else {
            true
        }
    };
    let translated_languages_filters = {
        let translated_languages = &param.translated_languages;
        if !translated_languages.is_empty() {
            filter_fn_via_translated_languages(item, translated_languages)
        } else {
            true
        }
    };
    let updated_at_since_filters = {
        if let Some(ref updated_at_since) = param.updated_at_since {
            filter_fn_via_updated_at_since(item, updated_at_since)
        } else {
            true
        }
    };
    let uploaders_filters = {
        let uploaders = &param.uploaders;
        if !uploaders.is_empty() {
            filter_fn_via_uploaders(item, uploaders)
        } else {
            true
        }
    };
    let volumes_filters = {
        let volumes = &param.volumes;
        if !volumes.is_empty() {
            filter_fn_via_volumes(item, volumes)
        } else {
            true
        }
    };
    filter_fn_excluded(item, param)
        && chapter_ids_filters
        && chapters_filters
        && content_rating_filters
        && created_at_since_filters
        && groups_filters
        && include_empty_pages_filters
        && manga_id_filters
        && original_languages_filters
        && publish_at_since_filters
        && title_filters
        && translated_languages_filters
        && updated_at_since_filters
        && uploaders_filters
        && volumes_filters
}
