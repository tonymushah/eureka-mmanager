use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::{ReferenceExpansionResource, RelationshipType};

pub fn map_fn_via_includes(
    mut item: CoverObject,
    includes: &[ReferenceExpansionResource],
) -> CoverObject {
    item.relationships.retain(|rel| match rel.type_ {
        RelationshipType::Manga => includes.contains(&ReferenceExpansionResource::Manga),
        RelationshipType::User => {
            includes.contains(&ReferenceExpansionResource::User)
                || includes.contains(&ReferenceExpansionResource::Creator)
        }
        _ => false,
    });
    item
}
