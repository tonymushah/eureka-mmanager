use api_core::{data_push::chapter::image::Mode, DirsOptions};
use emdx::PackageBuilder;
use uuid::Uuid;

use std::{fs::File, io::BufWriter};

fn main() -> anyhow::Result<()> {
    let dir_options = DirsOptions::default();
    let mut builder = PackageBuilder::new(dir_options);
    // add chapters data with the image saving mode
    builder.add_chapter(Uuid::new_v4(), Mode::DataSaver)?;
    // add manga data
    builder.add_manga(Uuid::new_v4())?;
    // add cover with it:s images
    builder.add_cover(Uuid::new_v4())?;
    let mut package = File::create("my-package.emdx")?;
    builder.build(BufWriter::new(&mut package))?;
    Ok(())
}
