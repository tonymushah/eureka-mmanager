use mangadex_api_input_types::cover::list::CoverListParam;
use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::{Language, RelationshipType};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{data_pulls::Validate, option_bool_match};

/// This parameter that allows you to filter a [`tokio_stream::Stream<Item = CoverObject>`] or a [`Iterator<Item = CoverObject>`]
/// via [`crate::prelude::IntoParamedFilteredStream`] for an async stream
/// or [`crate::prelude::IntoFiltered`] for an non-blocking iterator.
///
/// You can convert an [`CoverListParam`] into this
#[derive(Debug, Clone, Deserialize, Serialize, Default, Hash)]
pub struct CoverListDataPullFilterParams {
    pub manga_ids: Vec<Uuid>,
    pub uploader_ids: Vec<Uuid>,
    pub locales: Vec<Language>,
}

impl From<CoverListParam> for CoverListDataPullFilterParams {
    fn from(value: CoverListParam) -> Self {
        Self {
            manga_ids: value.manga_ids,
            uploader_ids: value.uploader_ids,
            locales: value.locales,
        }
    }
}

impl CoverListDataPullFilterParams {
    fn validate_manga_ids(&self, input: &CoverObject) -> Option<bool> {
        let manga = option_bool_match!(input.find_first_relationships(RelationshipType::Manga));
        let manga_ids = &self.manga_ids;
        if !manga_ids.is_empty() {
            Some(manga_ids.contains(&manga.id))
        } else {
            None
        }
    }
    fn validate_uploaders(&self, input: &CoverObject) -> Option<bool> {
        let uploader = option_bool_match!(input.find_first_relationships(RelationshipType::User));
        let uploaders = &self.uploader_ids;
        if !uploaders.is_empty() {
            Some(uploaders.contains(&uploader.id))
        } else {
            None
        }
    }
    fn validate_locale(&self, input: &CoverObject) -> Option<bool> {
        let locale = option_bool_match!(input.attributes.locale);
        let locales = &self.locales;
        if !locales.is_empty() {
            Some(locales.contains(&locale))
        } else {
            None
        }
    }
}

impl Validate<CoverObject> for CoverListDataPullFilterParams {
    fn is_valid(&self, input: &CoverObject) -> bool {
        let validations = vec![
            self.validate_locale(input),
            self.validate_manga_ids(input),
            self.validate_uploaders(input),
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
