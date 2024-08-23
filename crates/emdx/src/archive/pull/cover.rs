use std::{
    borrow::Cow,
    io::{self, Read},
    path::Path,
};

use mangadex_api_schema_rust::v5::CoverObject;
use tar::Entry;
use zstd::Decoder;

use crate::{PackageContents, ThisResult};

pub struct ArchiveCoverPull<'a, R>
where
    R: Read,
{
    pub(crate) entries: tar::Entries<'a, R>,
    pub(crate) package_contents: PackageContents,
}

impl<'a, R> ArchiveCoverPull<'a, R>
where
    R: Read,
{
    fn archive_entry_to_cover(&self, entry: Entry<'a, R>) -> ThisResult<CoverObject> {
        let options = self
            .package_contents
            .options
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_default();
        let dir_options = options
            .directories
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_default();
        if self
            .package_contents
            .data
            .values()
            .flat_map(|manga| &manga.covers)
            .map(|id| dir_options.covers_add(format!("{id}.cbor")))
            .any(|path| {
                let Ok(entry_path) = entry.path() else {
                    // eprintln!("invalid path");
                    return false;
                };
                // println!("{entry_path:?}");
                entry_path.as_ref() == AsRef::<Path>::as_ref(&path)
            })
        {
            if options.zstd_compressed_metadata {
                Ok(ciborium::from_reader(Decoder::new(entry)?)?)
            } else {
                Ok(ciborium::from_reader(entry)?)
            }
        } else {
            Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "the input path is not cover data cbor",
            )
            .into())
        }
    }
}

// Test if this fucking works
impl<'a, R> Iterator for ArchiveCoverPull<'a, R>
where
    R: Read,
{
    type Item = ThisResult<CoverObject>;
    fn next(&mut self) -> Option<Self::Item> {
        //println!("Pulling");
        let next = self.entries.next()?;
        match next {
            Ok(entry) => Some(self.archive_entry_to_cover(entry)),
            Err(err) => {
                // eprintln!("{err}");
                Some(Err(err.into()))
            }
        }
    }
}
