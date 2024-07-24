use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{self, BufWriter, Seek, Write},
    path::Path,
};

use api_core::{data_pulls::Pull, DirsOptions};
use mangadex_api_schema_rust::v5::{ChapterObject, CoverObject, MangaObject};
use tar::Builder as TarBuilder;
use tempfile::{tempdir, TempDir};
use uuid::Uuid;
use zstd::{stream::AutoFinishEncoder, Encoder};

use crate::{
    constants::{CHAPTER_CONTENT_FILE, COMPRESSION_LEVEL, CONTENTS_FILENAME},
    PackageContents,
};

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
}

impl<'a, W> BuilderInner<'a, W>
where
    W: Write,
{
    pub fn new(builder: Builder, writer: W) -> io::Result<Self> {
        let workdir = tempdir()?;
        let tar = TarBuilder::new(Encoder::new(writer, COMPRESSION_LEVEL)?.auto_finish());
        Ok(Self {
            workdir,
            tar,
            dir_options: builder.initial_dir_options,
            package_content: builder.contents,
            default_dir_options: Default::default(),
        })
    }
    fn create_workdir_file<P: AsRef<Path>>(&self, path: P) -> io::Result<File> {
        OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.workdir.path().join(path))
    }
    fn build_contents(&mut self) -> ThisResult<()> {
        let mut contents_file = self.create_workdir_file(CONTENTS_FILENAME)?;
        {
            let mut file_buf_writer = BufWriter::new(&mut contents_file);
            ciborium::into_writer(&self.package_content, &mut file_buf_writer)?;
            file_buf_writer.flush()?;
        }
        contents_file.rewind()?;
        self.tar
            .append_file(CONTENTS_FILENAME, &mut contents_file)?;
        Ok(())
    }
    fn build_chapter(&mut self, id: Uuid) -> ThisResult<()> {
        create_dir_all(self.workdir.path().join("chapters"))?;
        let mut content_files = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.workdir.path().join(format!("chapters/{id}.cbor")))?;
        {
            let chapter_data: ChapterObject = self.dir_options.pull(id)?;
            let mut writer = BufWriter::new(&mut content_files);
            ciborium::into_writer(&chapter_data, &mut writer)?;
            writer.flush()?;
        }
        content_files.rewind()?;
        let images = self
            .package_content
            .data
            .iter()
            .find_map(|(_, manga_data)| {
                manga_data.chapters.iter().find_map(|(chapter_id, images)| {
                    if *chapter_id == id {
                        Some(images)
                    } else {
                        None
                    }
                })
            })
            .ok_or(api_core::Error::Io(io::Error::from(
                io::ErrorKind::NotFound,
            )))?;
        for (filename, path) in images
            .data
            .iter()
            .map(|image| (image, self.dir_options.chapters_id_data_add(id).join(image)))
        {
            self.tar.append_file(
                self.default_dir_options
                    .chapters_id_data_add(id)
                    .join(filename),
                &mut File::open(path)?,
            )?;
        }
        for (filename, path) in images.data_saver.iter().map(|image| {
            (
                image,
                self.dir_options.chapters_id_data_saver_add(id).join(image),
            )
        }) {
            self.tar.append_file(
                self.default_dir_options
                    .chapters_id_data_saver_add(id)
                    .join(filename),
                &mut File::open(path)?,
            )?;
        }
        self.tar.append_file(
            self.default_dir_options
                .chapters_id_add(id)
                .join(CHAPTER_CONTENT_FILE),
            &mut content_files,
        )?;
        Ok(())
    }
    fn build_cover(&mut self, id: Uuid) -> ThisResult<()> {
        create_dir_all(self.workdir.path().join("covers"))?;
        let content_files_path = self.workdir.path().join(format!("covers/{id}.txt"));
        let mut content_files = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(content_files_path)?;
        //println!("pulling cover content");
        let cover = {
            let cover_data: CoverObject = self.dir_options.pull(id)?;
            let mut writer = BufWriter::new(&mut content_files);
            ciborium::into_writer(&cover_data, &mut writer)?;
            writer.flush()?;
            cover_data
        };
        content_files.rewind()?;
        //println!("writing cover image");
        self.tar.append_file(
            self.default_dir_options
                .cover_images_add(&cover.attributes.file_name),
            &mut {
                let image_path = self
                    .dir_options
                    .cover_images_add(&cover.attributes.file_name);
                File::open(image_path)?
            },
        )?;
        //println!("{:#?}", content_files.metadata()?);
        //println!("writing cover data");
        self.tar.append_file(
            self.default_dir_options.covers_add(format!("{id}.txr")),
            &mut content_files,
        )?;
        Ok(())
    }
    fn build_manga(&mut self, id: Uuid) -> ThisResult<()> {
        create_dir_all(self.workdir.path().join("mangas"))?;
        let mut content_files = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(self.workdir.path().join(format!("mangas/{id}.cbor")))?;
        let _ = {
            let manga_data: MangaObject = self.dir_options.pull(id)?;
            let mut writer = BufWriter::new(&mut content_files);
            ciborium::into_writer(&manga_data, &mut writer)?;
            writer.flush()?;
            manga_data
        };
        content_files.rewind()?;
        self.tar.append_file(
            self.default_dir_options.mangas_add(format!("{id}.cbor")),
            &mut content_files,
        )?;
        Ok(())
    }

    pub fn build(&mut self) -> ThisResult<()> {
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
        Ok(())
    }
}
