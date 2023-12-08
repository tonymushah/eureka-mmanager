use super::DefaultOffsetLimit;

// NOTE All download methods

#[cfg(feature = "actix_web")]
pub mod _download_manga_by_id;

#[cfg(feature = "actix_web")]
pub use _download_manga_by_id::download_manga_by_id;

#[cfg(feature = "actix_web")]
pub mod _download_manga_covers;

#[cfg(feature = "actix_web")]
pub use _download_manga_covers::download_manga_covers;

#[cfg(feature = "actix_web")]
pub mod _download_manga_cover;

#[cfg(feature = "actix_web")]
pub use _download_manga_cover::download_manga_cover;

#[cfg(feature = "actix_web")]
pub mod _download_manga_cover_quality;

#[cfg(feature = "actix_web")]
pub use _download_manga_cover_quality::download_manga_cover_quality;

#[cfg(feature = "actix_web")]
pub mod _download_cover;

#[cfg(feature = "actix_web")]
pub use _download_cover::download_cover;

#[cfg(feature = "actix_web")]
pub mod _download_cover_quality;

#[cfg(feature = "actix_web")]
pub use _download_cover_quality::download_cover_quality;

#[cfg(feature = "actix_web")]
pub mod _download_chapter_byid;

#[cfg(feature = "actix_web")]
pub use _download_chapter_byid::download_chapter_byid;

#[cfg(feature = "actix_web")]
pub mod _download_chapter_data_saver_byid;

#[cfg(feature = "actix_web")]
pub use _download_chapter_data_saver_byid::download_chapter_data_saver_byid;

#[cfg(feature = "actix_web")]
pub mod _download_chapter_data_byid;

#[cfg(feature = "actix_web")]
pub use _download_chapter_data_byid::download_chapter_data_byid;
