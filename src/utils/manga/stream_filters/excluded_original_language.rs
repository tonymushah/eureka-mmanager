use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::Language;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_excluded_original_language<'a, S>(
    stream: S,
    excluded_original_language: &'a [Language],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_excluded_original_language(item, excluded_original_language))
}

pub fn filter_fn_via_excluded_original_language<'a>(
    item: &'a MangaObject,
    excluded_original_language: &'a [Language],
) -> bool {
    !excluded_original_language.contains(&item.attributes.original_language)
}
