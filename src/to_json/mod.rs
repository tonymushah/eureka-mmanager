pub mod chapter;
pub mod manga;
use serde_json::json;
use mangadex_api::v5::schema::{self, Relationship};

pub fn Relationship_To_json(relationship : Relationship) -> serde_json::Value{
    return json!({
        "id": relationship.id,
        "type": relationship.type_,
        "related": relationship.related
    })
}

pub fn RefRelationship_To_json(relationship : &Relationship) -> serde_json::Value{
    return json!({
        "id": relationship.id,
        "type": relationship.type_,
        "related": relationship.related
    })
}