use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

pub fn filter_fn_via_uploader_ids<'a>(item: &'a CoverObject, uploader_ids: &'a [Uuid]) -> bool {
    item.relationships
        .iter()
        .any(|rel| rel.type_ == RelationshipType::User && uploader_ids.contains(&rel.id))
}
