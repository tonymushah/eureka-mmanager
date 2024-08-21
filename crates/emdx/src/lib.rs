pub mod archive;
pub mod builder;
pub mod constants;
pub mod contents;

pub use builder::Builder as PackageBuilder;
pub use contents::{PChapterObject, PMangaObject, PackageContents};

pub(crate) type ThisResult<T, E = api_core::Error> = Result<T, E>;
