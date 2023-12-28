use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::Language;

pub fn filter_fn_via_translated_languages<'a>(
    item: &'a ChapterObject,
    translated_languages: &'a [Language],
) -> bool {
    translated_languages.contains(&item.attributes.translated_language)
}
