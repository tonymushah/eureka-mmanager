use mangadex_api_schema_rust::v5::CoverObject;
use uuid::Uuid;

pub fn filter_fn_via_cover_ids<'a>(item: &'a CoverObject, cover_ids: &'a [Uuid]) -> bool {
    cover_ids.contains(&item.id)
}
