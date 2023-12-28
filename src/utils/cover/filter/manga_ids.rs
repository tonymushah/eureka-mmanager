use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

pub fn filter_fn_via_manga_ids<'a>(item: &'a CoverObject, manga_ids: &'a [Uuid]) -> bool {
    item.relationships
        .iter()
        .any(|rel| rel.type_ == RelationshipType::Manga && manga_ids.contains(&rel.id))
}
