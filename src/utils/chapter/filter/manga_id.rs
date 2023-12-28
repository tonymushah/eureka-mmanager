use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

pub fn filter_fn_via_manga_id<'a>(item: &'a ChapterObject, manga_id: &'a Uuid) -> bool {
    !item
        .find_relationships(RelationshipType::Manga)
        .iter()
        .any(|manga| manga.id == *manga_id)
}
