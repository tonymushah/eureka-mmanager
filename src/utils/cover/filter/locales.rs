use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::Language;

pub fn filter_fn_via_locales<'a>(item: &'a CoverObject, locales: &'a [Language]) -> bool {
    if let Some(ref locale) = item.attributes.locale {
        locales.contains(locale)
    } else {
        false
    }
}
