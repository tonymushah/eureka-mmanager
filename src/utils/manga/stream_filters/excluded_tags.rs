use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::TagSearchMode;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

pub fn filter_stream_via_excluded_tags<'a, S>(
    stream: S,
    excluded_tags: &'a [Uuid],
    mode: TagSearchMode,
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(move |item| filter_fn_via_excluded_tags(item, excluded_tags, mode))
}

pub fn filter_fn_via_excluded_tags<'a>(
    item: &'a MangaObject,
    excluded_tags: &'a [Uuid],
    mode: TagSearchMode,
) -> bool {
    let tag_ids: Vec<Uuid> = item.attributes.tags.iter().map(|tag| tag.id).collect();
    match mode {
        TagSearchMode::And => excluded_tags.iter().all(|tag| tag_ids.contains(tag)),
        TagSearchMode::Or => excluded_tags.iter().any(|tag| tag_ids.contains(tag)),
    }
}
