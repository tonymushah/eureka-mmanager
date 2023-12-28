use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::RelationshipType;
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

pub fn filter_stream_via_group<'a, S>(
    stream: S,
    group: &'a Uuid,
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.filter(|item| filter_fn_via_group(item, group))
}

pub fn filter_fn_via_group<'a>(item: &'a MangaObject, group: &'a Uuid) -> bool {
    item.relationships
        .iter()
        .any(|rel| rel.type_ == RelationshipType::ScanlationGroup && rel.id == *group)
}
