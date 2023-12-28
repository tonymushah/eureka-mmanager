use mangadex_api_schema_rust::{
    v5::{ChapterObject, MangaAttributes},
    ApiObjectNoRelationships,
};
use mangadex_api_types_rust::{ContentRating, RelationshipType};

pub fn filter_fn_via_content_rating<'a>(
    item: &'a ChapterObject,
    content_rating: &'a [ContentRating],
) -> bool {
    if let Some(manga) = item.find_first_relationships(RelationshipType::Manga) {
        if let Ok(manga) =
            std::convert::TryInto::<ApiObjectNoRelationships<MangaAttributes>>::try_into(
                manga.clone(),
            )
        {
            if let Some(ref content_rating_) = manga.attributes.content_rating {
                content_rating.contains(content_rating_)
            } else {
                false
            }
        } else {
            false
        }
    } else {
        false
    }
}
