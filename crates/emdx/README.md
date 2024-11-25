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
- The `content.cbor` file must contain all manga/cover/chapter/chapter-images registered in package. Other chapter/chapter-images/manga/cover data must be ignored
- Chapter/Manga/Cover metadata must be written in `cbor`

#### the `content.cbor` file

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

## Quick FAQ

1- Why [`cbor`][cbor] instead of `json`?

Mostly for personal choices but [`cbor`][cbor] has a lot of advantages compared to its `json`.
It's disk efficient and mostly stable.

2- Why [`zstd`][zstd] instead of a plain zip?

[cbor]: https://cbor.io/
[zstd]: https://github.com/facebook/zstd
