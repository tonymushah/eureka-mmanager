use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::MangaDexDateTime;

pub fn filter_fn_via_created_at_since<'a>(
    item: &'a ChapterObject,
    created_at_since: &'a MangaDexDateTime,
) -> bool {
    item.attributes.created_at.as_ref() > created_at_since.as_ref()
}
