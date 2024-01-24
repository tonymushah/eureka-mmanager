#[cfg(feature = "actix_web")]
use super::DefaultOffsetLimit;

#[cfg(feature = "actix_web")]
pub mod _hello;

#[cfg(feature = "actix_web")]
pub use _hello::hello;

#[cfg(feature = "actix_web")]
pub mod _find_manga_by_id;

#[cfg(feature = "actix_web")]
pub use _find_manga_by_id::find_manga_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_cover_by_id;

#[cfg(feature = "actix_web")]
pub use _find_cover_by_id::find_cover_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_cover_image_by_id;

#[cfg(feature = "actix_web")]
pub use _find_cover_image_by_id::find_cover_image_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_manga_cover_by_id;

#[cfg(feature = "actix_web")]
pub use _find_manga_cover_by_id::find_manga_cover_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_manga_covers_by_id;

#[cfg(feature = "actix_web")]
pub use _find_manga_covers_by_id::find_manga_covers_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_chapters_data_by_id;

#[cfg(feature = "actix_web")]
pub use _find_chapters_data_by_id::find_chapters_data_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_chapters_data_saver_by_id;

#[cfg(feature = "actix_web")]
pub use _find_chapters_data_saver_by_id::find_chapters_data_saver_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_chapters_data_img_by_id;

#[cfg(feature = "actix_web")]
pub use _find_chapters_data_img_by_id::find_chapters_data_img_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_chapters_data_saver_img_by_id;

#[cfg(feature = "actix_web")]
pub use _find_chapters_data_saver_img_by_id::find_chapters_data_saver_img_by_id;

#[cfg(feature = "actix_web")]
pub mod _find_chapter_by_id;

#[cfg(feature = "actix_web")]
pub use _find_chapter_by_id::find_chapter_by_id;

pub mod _find_all_downloaded_chapter;

#[cfg(feature = "actix_web")]
pub use _find_all_downloaded_chapter::find_all_downloaded_chapter;

pub mod _find_all_downloaded_manga;

#[cfg(feature = "actix_web")]
pub use _find_all_downloaded_manga::find_all_downloaded_manga;

#[cfg(feature = "actix_web")]
pub mod _find_manga_chapters_by_id;

#[cfg(feature = "actix_web")]
pub use _find_manga_chapters_by_id::find_manga_chapters_by_id;

#[cfg(feature = "actix_web")]
pub mod _aggregate_manga;

#[cfg(feature = "actix_web")]
pub use _aggregate_manga::aggregate_manga;
