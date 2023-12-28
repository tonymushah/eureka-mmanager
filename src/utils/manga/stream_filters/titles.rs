use mangadex_api_schema_rust::v5::MangaObject;
use regex::Regex;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_title<'a, S>(
    stream: S,
    title: &'a str,
) -> Option<impl Stream<Item = MangaObject> + 'a>
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    if let Ok(rgx) = Regex::new(title) {
        Some(stream.filter(move |item| filter_fn_via_title(item, rgx.clone())))
    } else {
        None
    }
}

pub fn filter_fn_via_title(item: &MangaObject, rgx: Regex) -> bool {
    let attributes = &item.attributes;
    if !attributes.title.values().any(|titles| rgx.is_match(titles)) {
        attributes
            .alt_titles
            .iter()
            .any(|titles_| titles_.values().any(|titles| rgx.is_match(titles)))
    } else {
        true
    }
}
