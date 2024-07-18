use std::path::PathBuf;

use mangadex_api_types_rust::RelationshipType;

#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    Io(#[from] std::io::Error),
    DirsOptionsVerification(#[from] crate::file_dirs::verification::DirsOptionsVerificationError),
    SerdeJson(#[from] serde_json::Error),
    #[error("Invalid file entry {0}")]
    InvalidFileName(PathBuf),
    #[error("Error when deserializing a .cbor file {0}")]
    CiboriumDeIo(#[from] ciborium::de::Error<std::io::Error>),
    #[error("Error when serializing a .cbor file {0}")]
    CiboriumSerIo(#[from] ciborium::ser::Error<std::io::Error>),
    #[error("Regex error {0}")]
    Regex(#[from] regex::Error),
    #[error("Missing Relationship {0:#?}")]
    MissingRelationships(Vec<RelationshipType>),
}
