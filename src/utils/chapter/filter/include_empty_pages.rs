use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::include_empty_pages::IncludeFuturePages;

pub fn filter_fn_via_include_empty_pages<'a>(
    item: &'a ChapterObject,
    include_empty_pages: &'a IncludeFuturePages,
) -> bool {
    if let IncludeFuturePages::Exclude = include_empty_pages {
        item.attributes.pages != 0
    } else {
        true
    }
}
