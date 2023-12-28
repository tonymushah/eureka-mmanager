use mangadex_api_schema_rust::v5::MangaObject;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

pub fn filter_stream_via_manga_list<'a, S>(
    stream: S,
    manga_ids: &'a [Uuid],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_manga_list(item, manga_ids))
}

pub fn filter_fn_via_manga_list<'a>(item: &'a MangaObject, manga_ids: &'a [Uuid]) -> bool {
    manga_ids.iter().any(|id| *id == item.id)
}
