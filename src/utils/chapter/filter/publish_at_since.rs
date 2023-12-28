use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::MangaDexDateTime;

pub fn filter_fn_via_publish_at_since<'a>(
    item: &'a ChapterObject,
    publish_at_since: &'a MangaDexDateTime,
) -> bool {
    if let Some(publish_at) = item.attributes.publish_at.as_ref() {
        publish_at.as_ref() > publish_at_since.as_ref()
    } else {
        false
    }
}
