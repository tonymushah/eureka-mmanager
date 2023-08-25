use super::DefaultOffsetLimit;

// NOTE All download methods

pub mod _download_manga_by_id;

pub use _download_manga_by_id::download_manga_by_id;

pub mod _download_manga_covers;

pub use _download_manga_covers::download_manga_covers;

pub mod _download_manga_cover;

pub use _download_manga_cover::download_manga_cover;

pub mod _download_manga_cover_quality;

pub use _download_manga_cover_quality::download_manga_cover_quality;

pub mod _download_cover;

pub use _download_cover::download_cover;

pub mod _download_cover_quality;

pub use _download_cover_quality::download_cover_quality;

pub mod _download_chapter_byid;

pub use _download_chapter_byid::download_chapter_byid;

pub mod _download_chapter_data_saver_byid;

pub use _download_chapter_data_saver_byid::download_chapter_data_saver_byid;

pub mod _download_chapter_data_byid;

pub use _download_chapter_data_byid::download_chapter_data_byid;
