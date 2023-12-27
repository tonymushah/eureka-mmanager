use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::RelationshipType;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

pub fn filter_stream_via_artists<'a, S>(
    stream: S,
    artists: &'a [Uuid],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_artists(item, artists))
}

pub fn filter_fn_via_artists<'a>(item: &'a MangaObject, artists: &'a [Uuid]) -> bool {
    item.relationships
        .iter()
        .any(|rel| rel.type_ == RelationshipType::Artist && artists.iter().any(|id| *id == rel.id))
}
