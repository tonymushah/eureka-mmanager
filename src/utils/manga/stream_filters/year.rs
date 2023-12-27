use mangadex_api_schema_rust::v5::MangaObject;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_year<'a, S>(
    stream: S,
    year: &'a u16,
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_year(item, year))
}

pub fn filter_fn_via_year<'a>(item: &'a MangaObject, year: &'a u16) -> bool {
    item.attributes.year.unwrap_or_default() == *year
}
