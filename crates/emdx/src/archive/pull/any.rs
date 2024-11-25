use std::{
    borrow::Cow,
    fs::File,
    io::{self, BufWriter, Read, Seek, Write},
    path::PathBuf,
};

use api_core::data_push::chapter::image::Mode as ChapterImageMode;
use mangadex_api_schema_rust::v5::{ChapterObject, CoverObject, MangaObject};
use regex::Regex;
use serde::de::DeserializeOwned;
use tar::{Entries, Entry};
use tempfile::tempfile;
use uuid::Uuid;
use zstd::Decoder;

use crate::{contents::options::PackageContentsOptions, PackageContents, ThisResult};

#[derive(Debug)]
pub enum PossibleEntryData {
    Manga(Box<MangaObject>),
    Chapter(Box<ChapterObject>),
    Cover(Box<CoverObject>),
    CoverImage {
        filename: String,
        file: File,
    },
    ChapterImage {
        filename: String,
        file: File,
        chapter: Uuid,
        mode: ChapterImageMode,
    },
    Any {
        tar_path: PathBuf,
        file: File,
    },
}

impl PossibleEntryData {
    pub fn try_clone(&self) -> io::Result<Self> {
        match self {
            PossibleEntryData::Manga(o) => Ok(Self::Manga(o.clone())),
            PossibleEntryData::Chapter(o) => Ok(Self::Chapter(o.clone())),
            PossibleEntryData::Cover(o) => Ok(Self::Cover(o.clone())),
            PossibleEntryData::CoverImage { filename, file } => Ok(Self::CoverImage {
                filename: filename.clone(),
                file: file.try_clone()?,
            }),
            PossibleEntryData::ChapterImage {
                filename,
                file,
                chapter,
                mode,
            } => Ok(Self::ChapterImage {
                filename: filename.clone(),
                file: file.try_clone()?,
                chapter: *chapter,
                mode: *mode,
            }),
            PossibleEntryData::Any {
                tar_path: filename,
                file,
            } => Ok(Self::Any {
                tar_path: filename.clone(),
                file: file.try_clone()?,
            }),
        }
    }
}

struct ArchiveAnyPullRegexes {
    manga: Regex,
    chapter: Regex,
    cover: Regex,
    cover_image: Regex,
    chapter_image: Regex,
}

impl ArchiveAnyPullRegexes {
    fn new(package_contents: &PackageContents) -> io::Result<Self> {
        let options = package_contents
            .options
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_default();
        let dir_options = options
            .directories
            .as_ref()
            .map(Cow::Borrowed)
            .unwrap_or_default();
        let chapter = Regex::new(
            dir_options.chapters_add(r"(?P<chapter_id>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/data.cbor")
            .as_os_str().to_str().ok_or(io::Error::new(io::ErrorKind::InvalidData, "invalid path input"))?
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let manga = Regex::new(
            dir_options.mangas_add(r"(?P<manga_id>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}).cbor")
            .as_os_str().to_str().ok_or(io::Error::new(io::ErrorKind::InvalidData, "invalid path input"))?
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let cover = Regex::new(
            dir_options.covers_add(r"(?P<cover_id>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12}).cbor")
            .as_os_str().to_str().ok_or(io::Error::new(io::ErrorKind::InvalidData, "invalid path input"))?
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let cover_image = Regex::new(
            dir_options
                .cover_images_add(r"(?P<filename>\w*.*)")
                .as_os_str()
                .to_str()
                .ok_or(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "invalid path input",
                ))?,
        )
        .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        let chapter_image = Regex::new(
            dir_options.chapters_add(r"(?P<chapter_id>[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{12})/(?P<mode>data|data-saver)/(?P<filename>\w*.*)")
            .as_os_str().to_str().ok_or(io::Error::new(io::ErrorKind::InvalidData, "invalid path input"))?
        ).map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;
        Ok(Self {
            manga,
            chapter,
            cover,
            chapter_image,
            cover_image,
        })
    }
}

enum PossibleEntryDataType {
    Manga,
    Chapter,
    Cover,
    CoverImage {
        filename: String,
    },
    ChapterImage {
        id: Uuid,
        filename: String,
        mode: ChapterImageMode,
    },
    Any,
}

pub struct ArchiveAnyPull<'a, R>
where
    R: 'a + Read,
{
    entries: Entries<'a, R>,
    package_contents: PackageContents,
    regexes: ArchiveAnyPullRegexes,
}

impl<'a, R> ArchiveAnyPull<'a, R>
where
    R: 'a + Read,
{
    pub(crate) fn new(
        entries: Entries<'a, R>,
        package_contents: PackageContents,
    ) -> io::Result<Self> {
        let regexes = ArchiveAnyPullRegexes::new(&package_contents)?;
        Ok(ArchiveAnyPull {
            entries,
            package_contents,
            regexes,
        })
    }
    fn entry_to_entry_type(&self, entry: &Entry<'a, R>) -> io::Result<PossibleEntryDataType> {
        let entry_path = entry.path()?;
        let path = entry_path.as_os_str().to_str().ok_or(io::Error::new(
            io::ErrorKind::InvalidInput,
            "Invalid path input",
        ))?;
        let type_: PossibleEntryDataType = if self.regexes.manga.is_match(path) {
            PossibleEntryDataType::Manga
        } else if self.regexes.chapter.is_match(path) {
            PossibleEntryDataType::Chapter
        } else if self.regexes.cover.is_match(path) {
            PossibleEntryDataType::Cover
        } else if let Some((filename, id, mode)) = self
            .regexes
            .chapter_image
            .captures(path)
            .and_then(|captures| {
                let filename: String = captures.name("filename")?.as_str().into();
                let id = Uuid::parse_str(captures.name("chapter_id")?.as_str()).ok()?;
                let mode = captures
                    .name("mode")
                    .map(|m| m.as_str())
                    .and_then(|m| match m {
                        "data" => Some(ChapterImageMode::Data),
                        "data-saver" => Some(ChapterImageMode::DataSaver),
                        _ => None,
                    })?;
                Some((filename, id, mode))
            })
        {
            PossibleEntryDataType::ChapterImage { id, filename, mode }
        } else if let Some(filename) =
            self.regexes
                .cover_image
                .captures(path)
                .and_then(|captures| {
                    let filename: String = captures.name("filename")?.as_str().into();
                    Some(filename)
                })
        {
            PossibleEntryDataType::CoverImage { filename }
        } else {
            PossibleEntryDataType::Any
        };
        Ok(type_)
    }
    fn get_package_contents_options(&self) -> Cow<'_, PackageContentsOptions> {
        self.package_contents.get_options()
    }
    fn deser_entry_metadata<O>(&self, entry: Entry<'a, R>) -> ThisResult<O>
    where
        O: DeserializeOwned,
    {
        let options = self.get_package_contents_options();
        if options.zstd_compressed_metadata {
            Ok(ciborium::from_reader(Decoder::new(entry)?)?)
        } else {
            Ok(ciborium::from_reader(entry)?)
        }
    }
    fn extract_images(&self, mut entry: Entry<'a, R>) -> ThisResult<File> {
        let mut temp = tempfile()?;
        {
            let mut temp_buf = BufWriter::new(&mut temp);
            let options = self.get_package_contents_options();
            if options.zstd_compressed_images {
                io::copy(&mut Decoder::new(&mut entry)?, &mut temp_buf)?;
            } else {
                io::copy(&mut entry, &mut temp_buf)?;
            }
            temp_buf.flush()?;
        }
        temp.rewind()?;
        Ok(temp)
    }
    fn extract_file(&self, mut entry: Entry<'a, R>) -> ThisResult<File> {
        let mut temp = tempfile()?;
        {
            let mut temp_buf = BufWriter::new(&mut temp);
            io::copy(&mut entry, &mut temp_buf)?;
            temp_buf.flush()?;
        }
        temp.rewind()?;
        Ok(temp)
    }
    fn entry_to_any(&self, entry: Entry<'a, R>) -> ThisResult<PossibleEntryData> {
        match self.entry_to_entry_type(&entry)? {
            PossibleEntryDataType::Manga => {
                Ok(PossibleEntryData::Manga(self.deser_entry_metadata(entry)?))
            }
            PossibleEntryDataType::Chapter => Ok(PossibleEntryData::Chapter(
                self.deser_entry_metadata(entry)?,
            )),
            PossibleEntryDataType::Cover => {
                Ok(PossibleEntryData::Cover(self.deser_entry_metadata(entry)?))
            }
            PossibleEntryDataType::CoverImage { filename } => Ok(PossibleEntryData::CoverImage {
                filename,
                file: self.extract_images(entry)?,
            }),
            PossibleEntryDataType::ChapterImage { id, filename, mode } => {
                Ok(PossibleEntryData::ChapterImage {
                    filename,
                    file: self.extract_images(entry)?,
                    chapter: id,
                    mode,
                })
            }
            PossibleEntryDataType::Any => {
                let path = entry.path()?.to_path_buf();
                Ok(PossibleEntryData::Any {
                    tar_path: path,
                    file: self.extract_file(entry)?,
                })
            }
        }
    }
}

impl<'a, R> Iterator for ArchiveAnyPull<'a, R>
where
    R: 'a + Read,
{
    type Item = ThisResult<PossibleEntryData>;
    fn next(&mut self) -> Option<Self::Item> {
        let entries_next = self
            .entries
            .next()?
            .map_err(|err| -> api_core::Error { err.into() });
        match entries_next {
            Ok(entry) => Some(self.entry_to_any(entry)),
            Err(err) => Some(Err(err)),
        }
    }
}
