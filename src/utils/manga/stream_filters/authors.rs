use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::RelationshipType;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

pub fn filter_stream_via_authors<'a, S>(
    stream: S,
    authors: &'a [Uuid],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_authors(item, authors))
}

pub fn filter_fn_via_authors<'a>(item: &'a MangaObject, authors: &'a [Uuid]) -> bool {
    item.relationships
        .iter()
        .any(|rel| rel.type_ == RelationshipType::Author && authors.iter().any(|id| *id == rel.id))
}
