use std::{
    io::{self, Read},
    path::Path,
};

use mangadex_api_schema_rust::v5::ChapterObject;
use tar::Entry;
use zstd::Decoder;

use crate::{PackageContents, ThisResult};

pub struct ArchiveChapterPull<'a, R>
where
    R: Read,
{
    pub(crate) entries: tar::Entries<'a, R>,
    pub(crate) package_contents: PackageContents,
}

impl<'a, R> ArchiveChapterPull<'a, R>
where
    R: Read,
{
    fn archive_entry_to_chapter(&self, entry: Entry<'a, R>) -> ThisResult<ChapterObject> {
        let options = self.package_contents.get_options();
        let dir_options = options.get_dirs();
        if self
            .package_contents
            .data
            .values()
            .flat_map(|manga| manga.chapters.keys().collect::<Vec<_>>())
            .map(|id| dir_options.chapters_add(format!("{id}/data.cbor")))
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
impl<R> Iterator for ArchiveChapterPull<'_, R>
where
    R: Read,
{
    type Item = ThisResult<ChapterObject>;
    fn next(&mut self) -> Option<Self::Item> {
        //println!("Pulling");
        let next = self.entries.next()?;
        match next {
            Ok(entry) => Some(self.archive_entry_to_chapter(entry)),
            Err(err) => {
                // eprintln!("{err}");
                Some(Err(err.into()))
            }
        }
    }
}
