use mangadex_api_schema_rust::v5::MangaObject;
use mangadex_api_types_rust::{ReferenceExpansionResource, RelationshipType};
use tokio_stream::{Stream, StreamExt};

pub fn map_stream_via_includes<'a, S>(
    stream: S,
    includes: &'a [ReferenceExpansionResource],
) -> impl Stream<Item = MangaObject> + 'a
where
    S: Stream<Item = MangaObject> + Unpin + 'a,
{
    stream.map(|o| map_fn_via_includes(o, includes))
}

pub fn map_fn_via_includes(
    mut o: MangaObject,
    includes: &[ReferenceExpansionResource],
) -> MangaObject {
    o.relationships.retain(|rel| match rel.type_ {
        RelationshipType::Manga => includes.contains(&ReferenceExpansionResource::Manga),
        RelationshipType::Artist => includes.contains(&ReferenceExpansionResource::Artist),
        RelationshipType::Author => includes.contains(&ReferenceExpansionResource::Author),
        RelationshipType::CoverArt => includes.contains(&ReferenceExpansionResource::CoverArt),
        RelationshipType::Creator => includes.contains(&ReferenceExpansionResource::Creator),
        _ => false,
    });
    o
}
