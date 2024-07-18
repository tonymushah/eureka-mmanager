#[derive(Debug, thiserror::Error)]
pub enum DirsOptionsVerificationError {
    #[error("The data dir doesn:t exist")]
    DataRoot,
    #[error("The history dir doesn:t exist")]
    History,
    #[error("The chapters dir doesn:t exist")]
    Chapters,
    #[error("The covers dir doesn:t exist")]
    Covers,
    #[error("The covers images dir doesn:t exist")]
    CoverImages,
    #[error("The mangas dir doesn't exist")]
    Mangas,
}
