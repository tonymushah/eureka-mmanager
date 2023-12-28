use mangadex_api_schema_rust::v5::ChapterObject;
use regex::Regex;

pub fn filter_fn_via_title<'a>(item: &'a ChapterObject, title: &'a str) -> bool {
    if let Some(ref chapter_title) = item.attributes.title {
        if let Ok(rgx) = Regex::new(title) {
            rgx.is_match(chapter_title)
        } else {
            false
        }
    } else {
        false
    }
}
