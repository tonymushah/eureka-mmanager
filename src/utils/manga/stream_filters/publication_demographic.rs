use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::Demographic;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_publication_demographic<'a, S>(
    stream: S,
    publication_demographic: &'a [Demographic],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_publication_demographic(item, publication_demographic))
}

pub fn filter_fn_via_publication_demographic<'a>(
    item: &'a MangaObject,
    publication_demographic: &'a [Demographic],
) -> bool {
    if let Some(ref demographic) = item.attributes.publication_demographic {
        publication_demographic.contains(demographic)
    } else {
        false
    }
}
