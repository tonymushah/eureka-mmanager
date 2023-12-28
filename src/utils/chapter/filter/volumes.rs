use mangadex_api_schema_rust::v5::ChapterObject;

pub fn filter_fn_via_volumes<'a>(item: &'a ChapterObject, volumes: &'a [String]) -> bool {
    match item.attributes.volume.as_ref() {
        None => false,
        Some(volume) => volumes.contains(volume),
    }
}
