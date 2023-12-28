use mangadex_api_input_types::cover::list::CoverListParam;
use mangadex_api_schema_rust::v5::CoverObject;

pub mod cover_ids;
pub mod includes;
pub mod locales;
pub mod manga_ids;
pub mod uploader_ids;

use self::{
    cover_ids::filter_fn_via_cover_ids, locales::filter_fn_via_locales,
    manga_ids::filter_fn_via_manga_ids, uploader_ids::filter_fn_via_uploader_ids,
};

pub fn filter<'a>(item: &'a CoverObject, param: &'a CoverListParam) -> bool {
    let cover_ids_filter = {
        let cover_ids = &param.cover_ids;
        if !cover_ids.is_empty() {
            filter_fn_via_cover_ids(item, cover_ids)
        } else {
            true
        }
    };
    let locales_filter = {
        let locales = &param.locales;
        if !locales.is_empty() {
            filter_fn_via_locales(item, locales)
        } else {
            true
        }
    };
    let uploader_ids_filter = {
        let uploader_ids = &param.uploader_ids;
        if !uploader_ids.is_empty() {
            filter_fn_via_uploader_ids(item, uploader_ids)
        } else {
            true
        }
    };
    let manga_ids_filter = {
        let manga_ids = &param.manga_ids;
        if !manga_ids.is_empty() {
            filter_fn_via_manga_ids(item, manga_ids)
        } else {
            true
        }
    };
    cover_ids_filter && locales_filter && uploader_ids_filter && manga_ids_filter
}
