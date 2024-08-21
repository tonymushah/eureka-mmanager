use std::{
    io::{self, BufRead, BufReader, Read, Seek},
    path::Path,
};

use zstd::Decoder;

use crate::{constants::CONTENTS_FILENAME, PackageContents, ThisResult};

pub struct Archive<'a, R>
where
    R: Seek + BufRead,
{
    contents: Option<PackageContents>,
    decoder: Decoder<'a, R>,
}

fn archive_contents_not_found_error() -> io::Error {
    io::Error::new(
        io::ErrorKind::NotFound,
        "this Archive Contents is not found",
    )
}

impl<'a, R> Archive<'a, R>
where
    R: Seek + BufRead,
{
    fn get_archive(&mut self, rewind: bool) -> io::Result<tar::Archive<&mut Decoder<'a, R>>> {
        if rewind {
            self.decoder.get_mut().rewind()?;
        }
        Ok(tar::Archive::new(&mut self.decoder))
    }
    fn use_tar_archive<F, O>(&mut self, rewind: bool, func: F) -> io::Result<O>
    where
        F: FnOnce(tar::Archive<&mut Decoder<'a, R>>) -> O,
    {
        Ok(func(self.get_archive(rewind)?))
    }
    fn seed_contents(&mut self) -> ThisResult<()> {
        let contents =
            self.use_tar_archive(true, |mut archive| -> ThisResult<PackageContents> {
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
    pub fn new<RD>(decoder: Decoder<'_, RD>) -> ThisResult<Archive<'_, RD>>
    where
        RD: BufRead + Seek,
    {
        let mut new_self = Archive {
            decoder,
            contents: None,
        };
        new_self.seed_contents()?;
        Ok(new_self)
    }
    pub fn from_reader<RD: Read + Seek>(reader: RD) -> ThisResult<Archive<'static, BufReader<RD>>> {
        let decoder = Decoder::new(reader)?;
        Self::new(decoder)
    }
    pub fn from_buf_read(buf_reader: R) -> ThisResult<Archive<'static, R>> {
        let decoder = Decoder::with_buffer(buf_reader)?;
        Self::new(decoder)
    }
}
