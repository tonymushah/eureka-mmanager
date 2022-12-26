use serde_json::json;
use mangadex_api::v5::schema::{self, MangaAttributes, TagAttributes};
use mangadex_api_schema::ApiObject;
use mangadex_api_types::MangaLink;
use mangadex_api_types::TagGroup;

pub fn Tags_to_json(tag: ApiObject<TagAttributes>) -> serde_json::Value{
    return json!({
        "id" : tag.id,
        "type" : tag.type_,
        "attributes" : json!({
            "name" : tag.attributes.name,
            "description" : tag.attributes.description,
            "group": tag.attributes.group,
            "version" : tag.attributes.version
        })
    })
}
pub fn RTags_to_json(tag: &ApiObject<TagAttributes>) -> serde_json::Value{
    return json!({
        "id" : tag.id,
        "type" : tag.type_,
        "attributes" : json!({
            "name" : tag.attributes.name,
            "description" : tag.attributes.description,
            "group": tag.attributes.group,
            "version" : tag.attributes.version
        })
    })
}
pub fn MangaAttr_to_json(mangaAttributes: MangaAttributes) -> serde_json::Value{
    let mut links = json!({});
    
    if mangaAttributes.links.is_none() == false {
        let liksd = mangaAttributes.links.expect("Error on execution");
        links = json!({
            "al" : liksd.anilist,
            "ap" : liksd.anime_planet,
            "bw" : liksd.book_walker.expect("Error on execution").0,
            "mu" : liksd.manga_updates.expect("Error on execution").0,
            "nu" : liksd.novel_updates.expect("Error on execution").0,
            "kt" : liksd.kitsu,
            "amz" : liksd.amazon,
            "ebj" : liksd.ebook_japan,
            "mal" : liksd.my_anime_list.expect("Error on execution").0,
            "cdj" : liksd.cd_japan,
            "engtl" : liksd.ebook_japan,
            "raw" : liksd.raw
        });
    }
    let mut tags: Vec<serde_json::Value> = Vec::new();
    for (index, value) in mangaAttributes.tags.iter().enumerate(){
        tags[index] = RTags_to_json(value);
    }
    return json!({
        "title": mangaAttributes.title,
        "alt_titles": mangaAttributes.alt_titles,
        "description": mangaAttributes.description,
        "is_locked": mangaAttributes.is_locked,
        "original_language": mangaAttributes.original_language,
        "last_volume": mangaAttributes.last_volume,
        "last_chapter": mangaAttributes.last_chapter,
        "publication_demographic": mangaAttributes.publication_demographic,
        "status": mangaAttributes.status,
        "year": mangaAttributes.year,
        "tags": tags,
        "links": links,
        "content_rating": mangaAttributes.content_rating,
        "chapter_numbers_reset_on_new_volume": mangaAttributes.chapter_numbers_reset_on_new_volume,
        "available_translated_languages": mangaAttributes.available_translated_languages,
        "state": mangaAttributes.state,
        "created_at": mangaAttributes.created_at,
        "updated_at": mangaAttributes.updated_at,
        "version": mangaAttributes.version
    })
}