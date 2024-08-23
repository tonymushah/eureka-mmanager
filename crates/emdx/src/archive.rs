pub mod pull;

use std::{
    fmt::Debug,
    io::{self, BufRead, BufReader, Read, Seek},
    path::Path,
};

use crate::utils::zstd_reader::Reader;
use mangadex_api_schema_rust::v5::{ChapterObject, CoverObject, MangaObject};
use pull::{
    chapter::ArchiveChapterPull, manga::ArchiveMangaPull, ArchiveAnyPull, ArchiveCoverPull,
};

use serde::de::DeserializeOwned;
use uuid::Uuid;
use zstd::stream::raw::Decoder;

use crate::{constants::CONTENTS_FILENAME, PackageContents, ThisResult};

type DecoderInner<'a, R> = Reader<R, Decoder<'a>>;

pub struct Archive<'a, R>
where
    R: Seek + BufRead,
{
    contents: Option<PackageContents>,
    tar_archive: Option<tar::Archive<DecoderInner<'a, R>>>,
}

impl<R> Debug for Archive<'_, R>
where
    R: Seek + BufRead,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Archive")
            .field("contents", &self.contents)
            .field("decoder", &"zstd::Decoder<'_, (some stream)>")
            .finish()
    }
}

fn archive_contents_not_found_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        "this Archive Contents is not found",
    )
}

fn tar_archive_not_found_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        "the underlying tar archive is not found",
    )
}

impl<'a, R> Seek for Archive<'a, R>
where
    R: Seek + BufRead,
{
    fn seek(&mut self, pos: io::SeekFrom) -> io::Result<u64> {
        let archive = self
            .tar_archive
            .take()
            .ok_or_else(tar_archive_not_found_error)?;
        let mut decoder = archive.into_inner();
        let res = decoder.reader_mut().seek(pos)?;
        decoder.reset()?;
        self.tar_archive.replace(tar::Archive::new(decoder));
        Ok(res)
    }
}

impl<R> Archive<'static, BufReader<R>>
where
    R: Seek + Read,
{
    pub fn from_reader(reader: R) -> ThisResult<Self> {
        Self::from_buf_read(BufReader::with_capacity(
            zstd::zstd_safe::CCtx::in_size(),
            reader,
        ))
    }
}

impl<'a, R> Archive<'a, R>
where
    R: Seek + BufRead,
{
    fn get_archive(&mut self, rewind: bool) -> io::Result<&mut tar::Archive<DecoderInner<'a, R>>> {
        if rewind {
            self.rewind()?;
        }
        self.tar_archive
            .as_mut()
            .ok_or_else(tar_archive_not_found_error)
    }
    fn use_tar_archive<F, O>(&mut self, rewind: bool, func: F) -> io::Result<O>
    where
        F: FnOnce(&mut tar::Archive<DecoderInner<'a, R>>) -> O,
    {
        Ok(func(self.get_archive(rewind)?))
    }
    fn seed_contents(&mut self) -> ThisResult<()> {
        let contents = self.use_tar_archive(true, |archive| -> ThisResult<PackageContents> {
            let mut content_file = archive
                .entries()?
                .flatten()
                .find(|file| {
                    if let Ok(ref path) = file.path() {
                        path == AsRef::<Path>::as_ref(CONTENTS_FILENAME)
                    } else {
                        false
                    }
                })
                .ok_or_else(archive_contents_not_found_error)?;
            Ok(ciborium::from_reader(&mut content_file)?)
        })??;
        self.contents.replace(contents);
        Ok(())
    }
    pub fn new(reader: R, decoder: Decoder<'_>) -> ThisResult<Archive<'_, R>> {
        let mut new_self = Archive {
            tar_archive: Some(tar::Archive::new(Reader::new(reader, decoder))),
            contents: None,
        };
        new_self.seed_contents()?;
        Ok(new_self)
    }

    pub fn from_buf_read(buf_reader: R) -> ThisResult<Archive<'static, R>> {
        Self::new(buf_reader, Decoder::new()?)
    }
    pub fn get_package_contents(&self) -> io::Result<&PackageContents> {
        self.contents
            .as_ref()
            .ok_or_else(archive_contents_not_found_error)
    }
    pub fn manga_pull(
        &mut self,
        rewind: bool,
    ) -> ThisResult<ArchiveMangaPull<DecoderInner<'a, R>>> {
        let package_contents = self.get_package_contents().cloned()?;
        let archive = self.get_archive(rewind)?;
        let entries = archive.entries()?;
        Ok(ArchiveMangaPull {
            entries,
            package_contents,
        })
    }
    pub fn cover_pull(
        &mut self,
        rewind: bool,
    ) -> ThisResult<ArchiveCoverPull<DecoderInner<'a, R>>> {
        let package_contents = self.get_package_contents().cloned()?;
        let archive = self.get_archive(rewind)?;
        let entries = archive.entries()?;
        Ok(ArchiveCoverPull {
            entries,
            package_contents,
        })
    }
    pub fn chapter_pull(
        &mut self,
        rewind: bool,
    ) -> ThisResult<ArchiveChapterPull<DecoderInner<'a, R>>> {
        let package_contents = self.get_package_contents().cloned()?;
        let archive = self.get_archive(rewind)?;
        let entries = archive.entries()?;
        Ok(ArchiveChapterPull {
            entries,
            package_contents,
        })
    }
    fn pull_data<O, F>(&mut self, rewind: bool, mut find: F) -> ThisResult<O>
    where
        O: DeserializeOwned,
        F: FnMut(&tar::Entry<DecoderInner<'a, R>>, &PackageContents) -> bool,
    {
        let package_contents = self.get_package_contents().cloned()?;
        self.use_tar_archive(rewind, |archive| {
            let options = package_contents.options.clone().unwrap_or_default();
            let entry = archive
                .entries()?
                .flatten()
                .find(|entry| find(entry, &package_contents))
                .ok_or(io::Error::new(
                    io::ErrorKind::NotFound,
                    "the manga/chapter/cover is not found in the archive",
                ))?;
            if options.zstd_compressed_metadata {
                Ok(ciborium::from_reader(zstd::stream::Decoder::new(entry)?)?)
            } else {
                Ok(ciborium::from_reader(entry)?)
            }
        })?
    }
    pub fn get_manga(&mut self, id: Uuid, rewind: bool) -> ThisResult<MangaObject> {
        self.pull_data(rewind, |entry, package_contents| {
            let dirs_options = package_contents
                .options
                .clone()
                .unwrap_or_default()
                .directories
                .unwrap_or_default();
            let Ok(path) = entry.path() else {
                return false;
            };
            path.as_ref() == AsRef::<Path>::as_ref(&dirs_options.mangas_add(format!("{id}.cbor")))
        })
        .map_err(|err| {
            let api_core::Error::Io(io_err) = err else {
                return err;
            };
            if io_err.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("the manga {id} is not found in the package"),
                )
                .into()
            } else {
                io_err.into()
            }
        })
    }
    pub fn get_cover(&mut self, id: Uuid, rewind: bool) -> ThisResult<CoverObject> {
        self.pull_data(rewind, |entry, package_contents| {
            let dirs_options = package_contents
                .options
                .clone()
                .unwrap_or_default()
                .directories
                .unwrap_or_default();
            let Ok(path) = entry.path() else {
                return false;
            };
            path.as_ref() == AsRef::<Path>::as_ref(&dirs_options.covers_add(format!("{id}.cbor")))
        })
        .map_err(|err| {
            let api_core::Error::Io(io_err) = err else {
                return err;
            };
            if io_err.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("the cover {id} is not found in the package"),
                )
                .into()
            } else {
                io_err.into()
            }
        })
    }
    pub fn get_chapter(&mut self, id: Uuid, rewind: bool) -> ThisResult<ChapterObject> {
        self.pull_data(rewind, |entry, package_contents| {
            let dirs_options = package_contents
                .options
                .clone()
                .unwrap_or_default()
                .directories
                .unwrap_or_default();
            let Ok(path) = entry.path() else {
                return false;
            };
            path.as_ref()
                == AsRef::<Path>::as_ref(&dirs_options.chapters_add(format!("{id}/data.cbor")))
        })
        .map_err(|err| {
            let api_core::Error::Io(io_err) = err else {
                return err;
            };
            if io_err.kind() == io::ErrorKind::NotFound {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    format!("the chapter {id} is not found in the package"),
                )
                .into()
            } else {
                io_err.into()
            }
        })
    }
    pub fn any_pull(&mut self, rewind: bool) -> ThisResult<ArchiveAnyPull<DecoderInner<'a, R>>> {
        let package_contents = self.get_package_contents().cloned()?;
        let archive = self.get_archive(rewind)?;
        let entries = archive.entries()?;
        Ok(ArchiveAnyPull::new(entries, package_contents)?)
    }
}
