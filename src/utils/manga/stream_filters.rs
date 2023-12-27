use mangadex_api_input_types::manga::list::MangaListParams;
use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::TagSearchMode;
use regex::Regex;

use self::{
    artists::filter_fn_via_artists, author_or_artist::filter_fn_via_author_or_artist,
    authors::filter_fn_via_authors,
    available_translated_language::filter_fn_via_available_translated_language,
    content_rating::filter_fn_via_content_rating, create_at_since::filter_fn_via_create_at_since,
    excluded_original_language::filter_fn_via_excluded_original_language,
    excluded_tags::filter_fn_via_excluded_tags, group::filter_fn_via_group,
    included_tags::filter_fn_via_included_tags, manga_list::filter_fn_via_manga_list,
    original_language::filter_fn_via_original_language,
    publication_demographic::filter_fn_via_publication_demographic, status::filter_fn_via_status,
    titles::filter_fn_via_title, updated_at_since::filter_fn_via_update_at_since,
    year::filter_fn_via_year,
};

pub mod artists;
pub mod author_or_artist;
pub mod authors;
pub mod available_translated_language;
pub mod content_rating;
pub mod create_at_since;
pub mod excluded_original_language;
pub mod excluded_tags;
pub mod group;
pub mod included_tags;
pub mod includes;
pub mod manga_list;
pub mod original_language;
pub mod publication_demographic;
pub mod status;
pub mod titles;
pub mod updated_at_since;
pub mod year;

pub fn filter_fn_tags<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    let excluded_tags_filter = {
        let excluded_tags = &params.excluded_tags;
        let mode = params.excluded_tags_mode.unwrap_or(TagSearchMode::Or);
        if !excluded_tags.is_empty() {
            filter_fn_via_excluded_tags(item, excluded_tags, mode)
        } else {
            true
        }
    };
    let included_tags_filter = {
        let included_tags = &params.excluded_tags;
        let mode = params.included_tags_mode.unwrap_or(TagSearchMode::And);
        if !included_tags.is_empty() {
            filter_fn_via_included_tags(item, included_tags, mode)
        } else {
            true
        }
    };
    excluded_tags_filter && included_tags_filter
}

pub fn filter_fn_since<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    let create_at_since_filter = {
        if let Some(create_at_since) = params.created_at_since.as_ref() {
            filter_fn_via_create_at_since(item, create_at_since)
        } else {
            true
        }
    };
    let updated_at_since_filter = {
        if let Some(updated_at_since) = params.updated_at_since.as_ref() {
            filter_fn_via_update_at_since(item, updated_at_since)
        } else {
            true
        }
    };
    create_at_since_filter && updated_at_since_filter
}

pub fn filter_fn_language<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    let available_translated_language_filter = {
        let available_translated_language = &params.available_translated_language;
        if !available_translated_language.is_empty() {
            filter_fn_via_available_translated_language(item, available_translated_language)
        } else {
            true
        }
    };
    let excluded_original_language_filter = {
        let excluded_original_language = &params.excluded_original_language;
        if !excluded_original_language.is_empty() {
            filter_fn_via_excluded_original_language(item, excluded_original_language)
        } else {
            true
        }
    };
    let original_language_filter = {
        let original_language = &params.original_language;
        if !original_language.is_empty() {
            filter_fn_via_original_language(item, original_language)
        } else {
            true
        }
    };
    available_translated_language_filter
        && excluded_original_language_filter
        && original_language_filter
}

pub fn filter_fn_misc<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    let content_rating_filter = {
        let content_rating = &params.content_rating;
        if !content_rating.is_empty() {
            filter_fn_via_content_rating(item, content_rating)
        } else {
            true
        }
    };
    let status_filter = {
        let status = &params.status;
        if !status.is_empty() {
            filter_fn_via_status(item, status)
        } else {
            true
        }
    };
    let year_filter = {
        if let Some(year) = params.year.as_ref() {
            filter_fn_via_year(item, year)
        } else {
            true
        }
    };
    let publication_demographic_filter = {
        let publication_demographic = &params.publication_demographic;
        if !publication_demographic.is_empty() {
            filter_fn_via_publication_demographic(item, publication_demographic)
        } else {
            true
        }
    };
    let titles_filter = {
        if let Some(titles) = params.title.as_ref() {
            if let Ok(rgx) = Regex::new(titles) {
                filter_fn_via_title(item, rgx)
            } else {
                true
            }
        } else {
            true
        }
    };
    content_rating_filter
        && status_filter
        && year_filter
        && publication_demographic_filter
        && titles_filter
}

pub fn filter_fn_author_artists<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    let authors_filter = {
        let authors = &params.authors;
        if !authors.is_empty() {
            filter_fn_via_authors(item, authors)
        } else {
            true
        }
    };
    let artists_filter = {
        let artists = &params.artists;
        if !artists.is_empty() {
            filter_fn_via_artists(item, artists)
        } else {
            true
        }
    };
    let author_or_artist_filter = {
        if let Some(ref author_or_artist) = params.author_or_artist {
            filter_fn_via_author_or_artist(item, author_or_artist)
        } else {
            true
        }
    };
    authors_filter && artists_filter && author_or_artist_filter
}

pub fn filter_fn_default<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    filter_fn_tags(item, params)
        && filter_fn_since(item, params)
        && filter_fn_language(item, params)
        && filter_fn_misc(item, params)
        && filter_fn_author_artists(item, params)
}

pub fn filter_via_manga_list<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    let manga_list_filter = {
        let manga_list = &params.manga_ids;
        if !manga_list.is_empty() {
            filter_fn_via_manga_list(item, manga_list)
        } else {
            true
        }
    };
    manga_list_filter && filter_fn_default(item, params)
}

pub fn filter_via_group<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    let group_filter = {
        if let Some(ref group) = params.group {
            filter_fn_via_group(item, group)
        } else {
            true
        }
    };
    group_filter && filter_fn_default(item, params)
}

pub fn filter<'a>(item: &'a MangaObject, params: &'a MangaListParams) -> bool {
    if !params.manga_ids.is_empty() {
        filter_via_manga_list(item, params)
    } else if params.group.is_some() {
        filter_via_group(item, params)
    } else {
        filter_fn_default(item, params)
    }
}
