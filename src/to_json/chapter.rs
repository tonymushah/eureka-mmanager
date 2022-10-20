use serde_json::json;
use mangadex_api::v5::schema::{self, ChapterAttributes};
use mangadex_api_schema::{ApiObject, v5::Relationship};

use super::RefRelationship_To_json;

pub fn ChapAttr_to_json(chapter: ChapterAttributes ) -> serde_json::Value{
    return json!({
        "title": chapter.title,
        "volume": chapter.volume,
        "chapter": chapter.chapter,
        "pages": chapter.pages,
        "translated_language": chapter.translated_language,
        "uploader": chapter.uploader,
        "external_url": chapter.external_url,
        "version": chapter.version,
        "created_at": chapter.created_at,
        "updated_at": chapter.updated_at,
        "publish_at": chapter.publish_at,
        "readable_at": chapter.readable_at
    })
}

pub fn chap_to_json(chapter: ApiObject<ChapterAttributes>) -> serde_json::Value{
    let mut relationships:Vec<serde_json::Value> = Vec::new();
    for (index, value) in chapter.relationships.iter().enumerate() {
        relationships[index] = RefRelationship_To_json(value);
    }
    return json!({
        "id" : chapter.id,
        "type" : chapter.type_,
        "attributes" : ChapAttr_to_json(chapter.attributes),
        "relationships" : relationships
    })
}