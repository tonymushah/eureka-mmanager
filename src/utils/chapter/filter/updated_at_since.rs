use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::MangaDexDateTime;

pub fn filter_fn_via_updated_at_since<'a>(
    item: &'a ChapterObject,
    updated_at_since: &'a MangaDexDateTime,
) -> bool {
    if let Some(updated_at) = item.attributes.updated_at.as_ref() {
        updated_at.as_ref() > updated_at_since.as_ref()
    } else {
        false
    }
}
