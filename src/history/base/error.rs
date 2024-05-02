use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum HistoryBaseError {
    #[error("Invalid Relationship Input {0}")]
    InvalidRelationship(RelationshipType),
    #[error("The value {0} already exists")]
    AlreadyExists(Uuid),
    #[error("The value {0} is not found")]
    NotFound(Uuid),
}
