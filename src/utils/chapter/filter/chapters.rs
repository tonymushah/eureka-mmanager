use mangadex_api_schema_rust::v5::ChapterObject;

pub fn filter_fn_via_chapters<'a>(item: &'a ChapterObject, chapters: &'a [String]) -> bool {
    match item.attributes.chapter.as_ref() {
        None => false,
        Some(chapter) => chapters.contains(chapter),
    }
}
