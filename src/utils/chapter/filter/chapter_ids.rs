use mangadex_api_schema_rust::v5::ChapterObject;
use uuid::Uuid;

pub fn filter_fn_via_chapter_ids<'a>(item: &'a ChapterObject, chapter_ids: &'a [Uuid]) -> bool {
    chapter_ids.contains(&item.id)
}
