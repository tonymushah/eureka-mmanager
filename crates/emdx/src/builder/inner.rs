use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{self, BufReader, BufWriter, Seek, Write},
    path::Path,
};

use api_core::{data_pulls::Pull, DirsOptions};
use image::{ImageFormat, ImageResult};
use mangadex_api_schema_rust::v5::{ChapterObject, CoverObject, MangaObject};
use serde::Serialize;
use tar::{Builder as TarBuilder, Header, HeaderMode};
use tempfile::{tempdir, TempDir};
use uuid::Uuid;
use zstd::{stream::AutoFinishEncoder, Encoder};

use crate::{
    constants::{CHAPTER_CONTENT_FILE, CONTENTS_FILENAME},
    PChapterObject, PackageContents,
};

enum BuilderInnerWriter<'a, W: Write> {
    Default(W),
    Encoder(AutoFinishEncoder<'a, W>),
}

impl<W: Write> BuilderInnerWriter<'_, W> {
    fn encoder(writer: W, compression_level: i32) -> io::Result<Self> {
        Ok(Self::Encoder(
            Encoder::new(writer, compression_level)?.auto_finish(),
        ))
    }
}

impl<W: Write> Write for BuilderInnerWriter<'_, W> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            BuilderInnerWriter::Default(w) => w.write(buf),
            BuilderInnerWriter::Encoder(e) => e.write(buf),
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            BuilderInnerWriter::Default(w) => w.flush(),
            BuilderInnerWriter::Encoder(e) => e.flush(),
        }
    }
}

use super::{Builder, ThisResult};
pub struct BuilderInner<'a, W>
where
    W: Write,
{
    package_content: PackageContents,
    workdir: TempDir,
    tar: TarBuilder<AutoFinishEncoder<'a, W>>,
    dir_options: DirsOptions,
    default_dir_options: DirsOptions,
    compression_level: i32,
    compress_image_to_jpeg: bool,
    header_mode: HeaderMode,
}

impl<W> BuilderInner<'_, W>
where
    W: Write,
{
    pub fn new(builder: Builder, writer: W) -> io::Result<Self> {
        let workdir = tempdir()?;
        let mut tar =
            TarBuilder::new(Encoder::new(writer, builder.compression_level)?.auto_finish());
        tar.mode(builder.header_mode);
        Ok(Self {
            compression_level: builder.compression_level,
            workdir,
            tar,
            dir_options: builder.initial_dir_options,
            package_content: builder.contents,
            default_dir_options: Default::default(),
            compress_image_to_jpeg: builder.compress_image_to_jpeg,
            header_mode: builder.header_mode,
        })
    }
    fn append_file<P: AsRef<Path>>(&mut self, path: P, file: &mut File) -> io::Result<()> {
        let mode = self.header_mode;
        let metadata = file.metadata()?;
        let mut header = Header::new_gnu();
        header.set_metadata_in_mode(&metadata, mode);
        let file_buffer = BufReader::new(file);
        self.tar.append_data(&mut header, path, file_buffer)?;
        Ok(())
    }
    fn create_workdir_file<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.workdir.path().join(path))
    }
    fn write_cbor_to_file<'b, C: Serialize>(
        &self,
        file: &'b mut File,
        content: &C,
    ) -> ThisResult<()> {
        let writer: BuilderInnerWriter<'b, &'b mut File> = if !self
            .package_content
            .options
            .as_ref()
            .map(|e| e.zstd_compressed_metadata)
            .unwrap_or_default()
        {
            BuilderInnerWriter::Default(file)
        } else {
            BuilderInnerWriter::encoder(file, self.compression_level)?
        };
        let mut file_buf_writer = BufWriter::new(writer);
        ciborium::into_writer(content, &mut file_buf_writer)?;
        file_buf_writer.flush()?;
        Ok(())
    }
    fn wctf<C: Serialize>(&self, file: &mut File, content: &C) -> ThisResult<()> {
        let mut file_buf_writer = BufWriter::new(file);
        ciborium::into_writer(content, &mut file_buf_writer)?;
        file_buf_writer.flush()?;
        Ok(())
    }
    fn build_contents(&mut self) -> ThisResult<()> {
        let mut contents_file = self.create_workdir_file(CONTENTS_FILENAME)?;
        self.wctf(&mut contents_file, &self.package_content)?;
        contents_file.rewind()?;
        self.append_file(CONTENTS_FILENAME, &mut contents_file)?;
        Ok(())
    }
    fn create_dir_all<P: AsRef<Path>>(&self, path: P) -> io::Result<()> {
        create_dir_all(self.workdir.path().join(path))
    }
    fn get_package_content_chapter_images(&self, chapter: Uuid) -> ThisResult<PChapterObject> {
        self.package_content
            .data
            .iter()
            .find_map(|(_, manga_data)| {
                manga_data.chapters.iter().find_map(|(chapter_id, images)| {
                    if *chapter_id == chapter {
                        Some(images.clone())
                    } else {
                        None
                    }
                })
            })
            .ok_or(api_core::Error::Io(io::Error::from(
                io::ErrorKind::NotFound,
            )))
    }
    fn append_image_file<P: AsRef<Path>>(
        &mut self,
        path: P,
        mut file: &mut File,
    ) -> io::Result<()> {
        if self
            .package_content
            .options
            .as_ref()
            .map(|e| e.zstd_compressed_images)
            .unwrap_or_default()
        {
            let mut temp = tempfile::tempfile()?;
            {
                let mut reader = BufReader::new(&mut file);
                let mut writer =
                    BufWriter::new(Encoder::new(&mut temp, self.compression_level)?.auto_finish());

                io::copy(&mut reader, &mut writer)?;
                writer.flush()?;
            }
            file.rewind()?;
            temp.rewind()?;
            self.append_file(path, &mut temp)?;
        } else {
            self.append_file(path, file)?;
        }
        Ok(())
    }
    fn convert_image_to_jpeg<P>(&self, path: P) -> ImageResult<(String, File)>
    where
        P: AsRef<Path>,
    {
        let current_format = ImageFormat::from_path(&path)?;
        let filename = path
            .as_ref()
            .file_name()
            .and_then(|e| e.to_str())
            .map(String::from)
            .ok_or(io::Error::new(
                io::ErrorKind::Unsupported,
                "Filename transport not found",
            ))?;
        let image = image::open(path)?;
        if current_format != ImageFormat::Gif {
            let new_filename = format!("{filename}.{}", ImageFormat::Jpeg.extensions_str()[0]);
            let mut file_output_path = self.create_workdir_file(&new_filename)?;
            {
                let mut file_out_buf = BufWriter::new(&mut file_output_path);
                image.write_to(&mut file_out_buf, ImageFormat::Jpeg)?;
                file_out_buf.flush()?;
            }
            file_output_path.rewind()?;
            Ok((new_filename, file_output_path))
        } else {
            Err(image::ImageError::IoError(io::Error::new(
                io::ErrorKind::Unsupported,
                "Gif can't be transformed into JPEG",
            )))
        }
    }
    fn append_chapter_images_data(
        &mut self,
        (id, images): (Uuid, &mut PChapterObject),
    ) -> io::Result<()> {
        let data = images
            .data
            .iter_mut()
            .map(|image| {
                let path = self.dir_options.chapters_id_data_add(id).join(&*image);
                (image, path)
            })
            .collect::<Vec<_>>();

        for (filename, path) in data {
            if self.compress_image_to_jpeg {
                if let Ok((new_filename, mut file)) = self.convert_image_to_jpeg(&path) {
                    *filename = new_filename;
                    self.append_image_file(
                        self.default_dir_options
                            .chapters_id_data_add(id)
                            .join(filename),
                        &mut file,
                    )?;
                } else {
                    self.append_image_file(
                        self.default_dir_options
                            .chapters_id_data_add(id)
                            .join(filename),
                        &mut File::open(&path)?,
                    )?;
                }
            } else {
                self.append_image_file(
                    self.default_dir_options
                        .chapters_id_data_add(id)
                        .join(filename),
                    &mut File::open(&path)?,
                )?;
            }
        }
        Ok(())
    }
    fn append_chapter_images_data_saver(
        &mut self,
        (id, images): (Uuid, &mut PChapterObject),
    ) -> io::Result<()> {
        let datas = images
            .data_saver
            .iter_mut()
            .map(|image| {
                let path = self
                    .dir_options
                    .chapters_id_data_saver_add(id)
                    .join(&*image);
                (image, path)
            })
            .collect::<Vec<_>>();
        for (filename, path) in datas {
            if self.compress_image_to_jpeg {
                if let Ok((new_filename, mut file)) = self.convert_image_to_jpeg(&path) {
                    *filename = new_filename;
                    self.append_image_file(
                        self.default_dir_options
                            .chapters_id_data_saver_add(id)
                            .join(filename),
                        &mut file,
                    )?;
                } else {
                    self.append_image_file(
                        self.default_dir_options
                            .chapters_id_data_saver_add(id)
                            .join(filename),
                        &mut File::open(&path)?,
                    )?;
                }
            } else {
                self.append_image_file(
                    self.default_dir_options
                        .chapters_id_data_saver_add(id)
                        .join(filename),
                    &mut File::open(&path)?,
                )?;
            }
        }
        Ok(())
    }
    fn pull_and_write_to_cbor<D: Serialize>(&mut self, id: Uuid, file: &mut File) -> ThisResult<D>
    where
        DirsOptions: Pull<D, Uuid, Error: Into<api_core::Error>>,
    {
        let data: D = self.dir_options.pull(id).map_err(|e| e.into())?;
        self.write_cbor_to_file(file, &data)?;
        Ok(data)
    }
    fn append_chapter_images(
        &mut self,
        (id, images): (Uuid, &mut PChapterObject),
    ) -> io::Result<()> {
        self.append_chapter_images_data((id, images))?;

        self.append_chapter_images_data_saver((id, images))?;
        Ok(())
    }
    fn build_chapter(&mut self, id: Uuid) -> ThisResult<()> {
        self.create_dir_all("chapters")?;
        let mut content_files = self.create_workdir_file(format!("chapters/{id}.cbor"))?;
        self.pull_and_write_to_cbor::<ChapterObject>(id, &mut content_files)?;
        content_files.rewind()?;
        let mut images = self.get_package_content_chapter_images(id)?;
        self.append_chapter_images((id, &mut images))?;
        self.append_file(
            self.default_dir_options
                .chapters_id_add(id)
                .join(CHAPTER_CONTENT_FILE),
            &mut content_files,
        )?;
        Ok(())
    }
    fn append_cover_image(&mut self, cover: &mut CoverObject) -> io::Result<()> {
        let filename = &mut cover.attributes.file_name;
        let image_path = self.dir_options.cover_images_add(&*filename);
        if self.compress_image_to_jpeg {
            if let Ok((new_filename, mut file)) = self.convert_image_to_jpeg(&image_path) {
                *filename = new_filename;
                self.append_image_file(
                    self.default_dir_options.cover_images_add(filename),
                    &mut file,
                )?;
            } else {
                self.append_image_file(
                    self.default_dir_options.cover_images_add(filename),
                    &mut File::open(image_path)?,
                )?;
            }
        } else {
            self.append_image_file(
                self.default_dir_options.cover_images_add(filename),
                &mut File::open(image_path)?,
            )?;
        }
        Ok(())
    }
    fn build_cover(&mut self, id: Uuid) -> ThisResult<()> {
        self.create_dir_all("covers")?;
        let mut content_files = self.create_workdir_file(format!("covers/{id}.cbor"))?;
        //println!("pulling cover content");
        let mut cover: CoverObject = self.dir_options.pull(id)?;
        //println!("writing cover image");
        self.append_cover_image(&mut cover)?;
        self.write_cbor_to_file(&mut content_files, &cover)?;
        content_files.rewind()?;
        //println!("{:#?}", content_files.metadata()?);
        //println!("writing cover data");
        self.append_file(
            self.default_dir_options.covers_add(format!("{id}.cbor")),
            &mut content_files,
        )?;
        Ok(())
    }
    fn build_manga(&mut self, id: Uuid) -> ThisResult<()> {
        self.create_dir_all("mangas")?;
        let mut content_files = self.create_workdir_file(format!("mangas/{id}.cbor"))?;
        self.pull_and_write_to_cbor::<MangaObject>(id, &mut content_files)?;
        content_files.rewind()?;
        self.append_file(
            self.default_dir_options.mangas_add(format!("{id}.cbor")),
            &mut content_files,
        )?;
        Ok(())
    }

    pub fn build(mut self) -> ThisResult<PackageContents> {
        for (manga_id, manga_data) in self.package_content.data.clone() {
            // println!("writing {manga_id}");
            for cover_id in &manga_data.covers {
                //println!("writing cover {cover_id}");
                self.build_cover(*cover_id)?;
                //println!("builded cover");
            }
            //println!("writing {manga_id} manga data");
            self.build_manga(manga_id)?;
            //println!("writing {manga_id}");
            for chapter in manga_data.chapters.keys() {
                //println!("writing {chapter} chapter");
                self.build_chapter(*chapter)?;
                //println!("writed {chapter} chapter");
            }
        }
        //println!("writing contents");
        self.build_contents()?;
        //println!("writed");
        Ok(self.package_content)
    }
}
