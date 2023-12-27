use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::ContentRating;
use tokio_stream::{Stream, StreamExt};

pub fn filter_stream_via_content_rating<'a, S>(
    stream: S,
    content_rating: &'a [ContentRating],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_content_rating(item, content_rating))
}

pub fn filter_fn_via_content_rating<'a>(
    item: &'a MangaObject,
    content_rating: &'a [ContentRating],
) -> bool {
    if let Some(ref rating) = item.attributes.content_rating {
        content_rating.contains(rating)
    } else {
        false
    }
}
