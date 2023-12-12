#[cfg(feature = "actix_web")]
pub mod _delete_chapter_by_id;

#[cfg(feature = "actix_web")]
pub use _delete_chapter_by_id::delete_chapter_by_id;

#[cfg(feature = "actix_web")]
pub mod _delete_manga_chapters_by_id;

#[cfg(feature = "actix_web")]
pub use _delete_manga_chapters_by_id::delete_manga_chapters_by_id;
