use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::MangaStatus;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_status<'a, S>(
    stream: S,
    status: &'a [MangaStatus],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_status(item, status))
}

pub fn filter_fn_via_status<'a>(item: &'a MangaObject, status: &'a [MangaStatus]) -> bool {
    status.contains(&item.attributes.status)
}
