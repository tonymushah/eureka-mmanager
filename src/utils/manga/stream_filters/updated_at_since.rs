use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::MangaDexDateTime;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_update_at_since<'a, S>(
    stream: S,
    updated_at_since: &'a MangaDexDateTime,
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_update_at_since(item, updated_at_since))
}

pub fn filter_fn_via_update_at_since<'a>(
    item: &'a MangaObject,
    updated_at_since: &'a MangaDexDateTime,
) -> bool {
    if let Some(ref updated_at) = item.attributes.updated_at {
        updated_at.as_ref() < updated_at_since.as_ref()
    } else {
        false
    }
}
