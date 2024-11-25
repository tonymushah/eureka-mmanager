# emdx

`emdx` is a package system designed to store title/cover/chapter from MangaDex.org for long term.

## Design goals

Let's first describes the goals, specification.

### Goals

The initial idea of `emdx` is to allow a MangaDex based app users (that directly or indirectly uses the ) to **extract** and **store** titles, their covers, and theirs chapters for the long term
and also **restore** it back to the main app at any moment.

### Specification

Every `emdx` package should follow these rules:

- The `.emdx` should be a readable `.tar.zstd` file
- The file must contain an `content.cbor` entry
- The `contents.cbor` file must contain all manga/cover/chapter/chapter-images registered in package. Other chapter/chapter-images/manga/cover data must be ignored
- Chapter/Manga/Cover metadata must be written in `cbor`

#### the `contents.cbor` file

You can say that it is the hearth of a `.emdx` package.
It contains what is inside of the package: the file structure configuration, the manga/cover/chapter/chapter-images that is inside and many other options.

Example of what should be inside of a `content.cbor` file:

```jsonc
/// I use json here to make it clear to see
{
    "options": {
        /// This folllow the [`core::DirsOptions`] specification and also optional
        "directories": {
            /// the main data directories
            "data_dir": "data",
            "chapters": "chapters",
            "mangas": "manga",
            "covers": "covers"
        },
        /// zstd
        "zstd_compressed_metadata": false,
        "zstd_compressed_images": false
    },
    "data": {
        // Manga ID
        "a742e120-ab18-11ef-987b-ec21e559732b": {
            /// Cover ids
            "covers": [
                "1e0b0e02-ab1b-11ef-b48c-ec21e559732b",
                "24b223f8-ab1b-11ef-a693-ec21e559732b"
            ],
            /// Chapter data
            "chapters":{ 
                /// Chapter id
                "d4a3d364-ab1c-11ef-a36a-ec21e559732b": {
                    /// normal chapter images
                    "data": [],
                    /// data-saver chapter images
                    "data_saver": [
                        "1.jpg",
                        "2.jpg",
                        "3.jpg"
                    ]
                }
            }
        },
        "bd939b0e-ab18-11ef-b0fa-ec21e559732b": {
            /* More manga related data */
        },
        "d18ab430-ab18-11ef-9275-ec21e559732b": {
            /* More manga related data */
        }
    }
}
```

## The package itself

This crate allows to you to extract and restore an `.emdx` package from an existing `eureka-mmanager`.

### Installation

Add the `eureka-mmanager-core` and `emdx` in your `Cargo.toml` dependecies

```toml
[dependencies]
eureka-mmanager-core = "0.1"
emdx = "0.1"
```

### Examples

#### Getting chapter from an archive

```rust
use std::fs::File;

use emdx::Archive;

fn main() -> anyhow::Result<()> {
    let mut archive = File::open("your_package.emdx")?;
    let mut emdx_package = Archive::from_reader(&mut archive)?;
    for chapter in emdx_package.chapter_pull(true)?.flatten() {
        println!("has chapter {}", chapter.id);
    }
    Ok(())
}
```

### Making a emdx package

```rust
use eureka_mmanager_core::{data_push::chapter::image::Mode, DirsOptions};
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
```
