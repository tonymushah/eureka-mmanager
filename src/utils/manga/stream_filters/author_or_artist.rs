use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::RelationshipType;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

pub fn filter_stream_via_author_or_artist<'a, S>(
    stream: S,
    author_or_artist: &'a Uuid,
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_author_or_artist(item, author_or_artist))
}

pub fn filter_fn_via_author_or_artist<'a>(
    item: &'a MangaObject,
    author_or_artist: &'a Uuid,
) -> bool {
    item.relationships.iter().any(|rel| {
        (rel.type_ == RelationshipType::Author || rel.type_ == RelationshipType::Artist)
            && rel.id == *author_or_artist
    })
}
