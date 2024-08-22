use std::{
    io::{self, Read},
    path::Path,
};

use mangadex_api_schema_rust::v5::MangaObject;
use tar::Entry;
use zstd::Decoder;

use crate::{PackageContents, ThisResult};

pub struct ArchiveMangaPull<'a, R>
where
    R: Read,
{
    pub(crate) entries: tar::Entries<'a, R>,
    pub(crate) package_contents: PackageContents,
}

impl<'a, R> ArchiveMangaPull<'a, R>
where
    R: Read,
{
    fn archive_entry_to_manga(&self, entry: Entry<'a, R>) -> ThisResult<MangaObject> {
        let options = self.package_contents.options.clone().unwrap_or_default();
        let dir_options = options.directories.unwrap_or_default();
        if self
            .package_contents
            .data
            .keys()
            .map(|id| dir_options.mangas_add(format!("{id}.cbor")))
            .any(|path| {
                let Ok(entry_path) = entry.path() else {
                    eprintln!("invalid path");
                    return false;
                };
                println!("{entry_path:?}");
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
                "the input path is not manga data cbor",
            )
            .into())
        }
    }
}

// Test if this fucking works
impl<'a, R> Iterator for ArchiveMangaPull<'a, R>
where
    R: Read,
{
    type Item = ThisResult<MangaObject>;
    fn next(&mut self) -> Option<Self::Item> {
        println!("Pulling");
        let next = self.entries.next()?;
        match next {
            Ok(entry) => Some(self.archive_entry_to_manga(entry)),
            Err(err) => {
                eprintln!("{err}");
                Some(Err(err.into()))
            }
        }
    }
}
