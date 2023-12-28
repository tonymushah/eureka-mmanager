use mangadex_api_schema_rust::{
    v5::{ChapterObject, MangaAttributes},
    ApiObjectNoRelationships,
};
use mangadex_api_types_rust::{Language, RelationshipType};

pub fn filter_fn_via_original_languages<'a>(
    item: &'a ChapterObject,
    original_languages: &'a [Language],
) -> bool {
    if let Some(manga) = item.find_first_relationships(RelationshipType::Manga) {
        if let Ok(manga) =
            std::convert::TryInto::<ApiObjectNoRelationships<MangaAttributes>>::try_into(
                manga.clone(),
            )
        {
            original_languages.contains(&manga.attributes.original_language)
        } else {
            false
        }
    } else {
        false
    }
}
