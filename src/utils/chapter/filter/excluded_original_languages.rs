use mangadex_api_schema_rust::{
    v5::{ChapterObject, MangaAttributes},
    ApiObjectNoRelationships,
};
use mangadex_api_types_rust::{Language, RelationshipType};

pub fn filter_fn_via_excluded_original_languages<'a>(
    item: &'a ChapterObject,
    excluded_original_language: &'a [Language],
) -> bool {
    if let Some(manga) = item.find_first_relationships(RelationshipType::Manga) {
        if let Ok(manga) =
            std::convert::TryInto::<ApiObjectNoRelationships<MangaAttributes>>::try_into(
                manga.clone(),
            )
        {
            !excluded_original_language.contains(&manga.attributes.original_language)
        } else {
            true
        }
    } else {
        true
    }
}
