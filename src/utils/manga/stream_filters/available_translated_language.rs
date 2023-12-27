use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::Language;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_available_translated_language<'a, S>(
    stream: S,
    available_translated_language: &'a [Language],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| {
        filter_fn_via_available_translated_language(item, available_translated_language)
    })
}

pub fn filter_fn_via_available_translated_language<'a>(
    item: &'a MangaObject,
    available_translated_language: &'a [Language],
) -> bool {
    available_translated_language.iter().any(|lang| {
        item.attributes
            .available_translated_languages
            .contains(lang)
    })
}
