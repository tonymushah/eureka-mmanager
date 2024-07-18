#[derive(Debug, thiserror::Error)]
#[error(transparent)]
pub enum Error {
    Io(#[from] std::io::Error),
    DirsOptionsVerification(#[from] crate::file_dirs::verification::DirsOptionsVerificationError),
    SerdeJson(#[from] serde_json::Error),
}
