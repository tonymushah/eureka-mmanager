// NOTE update api data

#[cfg(feature = "actix_web")]
pub mod _update_cover_by_id;

#[cfg(feature = "actix_web")]
pub use _update_cover_by_id::update_cover_by_id;

#[cfg(feature = "actix_web")]
pub mod _update_chapter_by_id;

#[cfg(feature = "actix_web")]
pub use _update_chapter_by_id::update_chapter_by_id;

#[cfg(feature = "actix_web")]
pub mod _patch_all_chapter;

#[cfg(feature = "actix_web")]
pub use _patch_all_chapter::patch_all_chapter;

#[cfg(feature = "actix_web")]
pub mod _patch_all_chapter_manga;

#[cfg(feature = "actix_web")]
pub use _patch_all_chapter_manga::patch_all_chapter_manga;

#[cfg(feature = "actix_web")]
pub mod _update_chapter_manga_by_id;

#[cfg(feature = "actix_web")]
pub use _update_chapter_manga_by_id::update_chapter_manga_by_id;

#[cfg(feature = "actix_web")]
pub mod _patch_all_manga_cover;

#[cfg(feature = "actix_web")]
pub use _patch_all_manga_cover::patch_all_manga_cover;
