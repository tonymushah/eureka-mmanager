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
    o.relationships.iter_mut().for_each(|rel| match rel.type_ {
        RelationshipType::Manga => {
            if !includes.contains(&ReferenceExpansionResource::Manga) {
                rel.attributes.take();
            }
        }
        RelationshipType::Artist => {
            if !includes.contains(&ReferenceExpansionResource::Artist) {
                rel.attributes.take();
            }
        }
        RelationshipType::Author => {
            if !includes.contains(&ReferenceExpansionResource::Author) {
                rel.attributes.take();
            }
        }
        RelationshipType::CoverArt => {
            if !includes.contains(&ReferenceExpansionResource::CoverArt) {
                rel.attributes.take();
            }
        }
        RelationshipType::Creator => {
            if !includes.contains(&ReferenceExpansionResource::Creator) {
                rel.attributes.take();
            }
        }
        _ => {}
    });
    o
}
