use std::{
    fs::{create_dir_all, File, OpenOptions},
    io::{self, BufWriter, Seek, Write},
    path::Path,
};

use api_core::{data_pulls::Pull, DirsOptions};
use mangadex_api_schema_rust::v5::{ChapterObject, CoverObject, MangaObject};
use serde::Serialize;
use tar::Builder as TarBuilder;
use tempfile::{tempdir, TempDir};
use uuid::Uuid;
use zstd::{stream::AutoFinishEncoder, Encoder};

use crate::{
    constants::{CHAPTER_CONTENT_FILE, COMPRESSION_LEVEL, CONTENTS_FILENAME},
    PChapterObject, PackageContents,
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
    fn write_cbor_to_file<C: Serialize>(&self, file: &mut File, content: &C) -> ThisResult<()> {
        let mut file_buf_writer = BufWriter::new(file);
        ciborium::into_writer(content, &mut file_buf_writer)?;
        file_buf_writer.flush()?;
        Ok(())
    }
    fn build_contents(&mut self) -> ThisResult<()> {
        let mut contents_file = self.create_workdir_file(CONTENTS_FILENAME)?;
        self.write_cbor_to_file(&mut contents_file, &self.package_content)?;
        contents_file.rewind()?;
        self.tar
            .append_file(CONTENTS_FILENAME, &mut contents_file)?;
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
    fn append_chapter_images_data(
        &mut self,
        (id, images): (Uuid, &PChapterObject),
    ) -> io::Result<()> {
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
        Ok(())
    }
    fn append_chapter_images_data_saver(
        &mut self,
        (id, images): (Uuid, &PChapterObject),
    ) -> io::Result<()> {
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
    fn append_chapter_images(&mut self, data: (Uuid, &PChapterObject)) -> io::Result<()> {
        self.append_chapter_images_data(data)?;

        self.append_chapter_images_data_saver(data)?;
        Ok(())
    }
    fn build_chapter(&mut self, id: Uuid) -> ThisResult<()> {
        self.create_dir_all("chapters")?;
        let mut content_files = self.create_workdir_file(format!("chapters/{id}.cbor"))?;
        self.pull_and_write_to_cbor::<ChapterObject>(id, &mut content_files)?;
        content_files.rewind()?;
        let images = self.get_package_content_chapter_images(id)?;
        self.append_chapter_images((id, &images))?;
        self.tar.append_file(
            self.default_dir_options
                .chapters_id_add(id)
                .join(CHAPTER_CONTENT_FILE),
            &mut content_files,
        )?;
        Ok(())
    }
    fn append_cover_image(&mut self, cover: &CoverObject) -> io::Result<()> {
        self.tar.append_file(
            self.default_dir_options
                .cover_images_add(&cover.attributes.file_name),
            &mut {
                let image_path = self
                    .dir_options
                    .cover_images_add(&cover.attributes.file_name);
                File::open(image_path)?
            },
        )
    }
    fn build_cover(&mut self, id: Uuid) -> ThisResult<()> {
        self.create_dir_all("covers")?;
        let mut content_files = self.create_workdir_file(format!("covers/{id}.cbor"))?;
        //println!("pulling cover content");
        let cover = self.pull_and_write_to_cbor::<CoverObject>(id, &mut content_files)?;
        content_files.rewind()?;
        //println!("writing cover image");
        self.append_cover_image(&cover)?;
        //println!("{:#?}", content_files.metadata()?);
        //println!("writing cover data");
        self.tar.append_file(
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
        self.tar.append_file(
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
