use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::MangaDexDateTime;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_create_at_since<'a, S>(
    stream: S,
    create_at_since: &'a MangaDexDateTime,
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_create_at_since(item, create_at_since))
}

pub fn filter_fn_via_create_at_since<'a>(
    item: &'a MangaObject,
    create_at_since: &'a MangaDexDateTime,
) -> bool {
    item.attributes.created_at.as_ref() < create_at_since.as_ref()
}
