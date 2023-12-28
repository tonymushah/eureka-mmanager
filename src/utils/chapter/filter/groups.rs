use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

pub fn filter_fn_via_groups<'a>(item: &'a ChapterObject, groups: &'a [Uuid]) -> bool {
    item.find_relationships(RelationshipType::ScanlationGroup)
        .iter()
        .any(|group| groups.contains(&group.id))
}
