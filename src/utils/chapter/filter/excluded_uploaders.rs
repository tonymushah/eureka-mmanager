use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

pub fn filter_fn_via_excluded_uploaders<'a>(
    item: &'a ChapterObject,
    excluded_uploaders: &'a [Uuid],
) -> bool {
    !item
        .find_relationships(RelationshipType::User)
        .iter()
        .any(|uploader| excluded_uploaders.contains(&uploader.id))
}
