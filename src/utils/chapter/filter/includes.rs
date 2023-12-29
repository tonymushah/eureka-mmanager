use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::{ReferenceExpansionResource, RelationshipType};

pub fn map_fn_via_includes(
    mut item: ChapterObject,
    includes: &[ReferenceExpansionResource],
) -> ChapterObject {
    item.relationships
        .iter_mut()
        .for_each(|rel| match rel.type_ {
            RelationshipType::Manga => {
                if !includes.contains(&ReferenceExpansionResource::Manga) {
                    rel.attributes.take();
                }
            }
            RelationshipType::User => {
                if !(includes.contains(&ReferenceExpansionResource::User)
                    || includes.contains(&ReferenceExpansionResource::Creator))
                {
                    rel.attributes.take();
                }
            }
            RelationshipType::ScanlationGroup => {
                if !includes.contains(&ReferenceExpansionResource::ScanlationGroup) {
                    rel.attributes.take();
                }
            }
            _ => {}
        });
    item
}
