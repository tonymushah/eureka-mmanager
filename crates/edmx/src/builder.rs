use api_core::DirsOptions;

use crate::PackageContents;

pub struct Builder {
    initial_dir_options: DirsOptions,
    contents: PackageContents,
}
