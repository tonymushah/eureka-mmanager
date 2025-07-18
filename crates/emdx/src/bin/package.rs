use std::{
    fs::File,
    io::{BufWriter, Write},
    thread,
    time::Instant,
};

use api_core::{
    data_pulls::{
        chapter::ChapterListDataPullFilterParams, manga::MangaListDataPullFilterParams,
        IntoFiltered,
    },
    DirsOptions,
};
use emdx::PackageBuilder;
use mangadex_api_types_rust::Language;

mod package {
    use super::*;
    pub const DICT_FILE: &str = "target/fuufu-ijou-v8-v9-en.zstd.dict";
    fn dict(builder: &PackageBuilder) {
        let start = Instant::now();
        let mut output_file = File::create(DICT_FILE).unwrap();
        let mut output_file_buf_writer = BufWriter::new(&mut output_file);
        output_file_buf_writer
            .write_all(&builder.create_dict(16_000_000).unwrap())
            .unwrap();
        output_file_buf_writer.flush().unwrap();
        let build_time = Instant::now() - start;
        println!("Dict Build Time: {} s", build_time.as_secs_f64());
    }

    pub const NORMAL_FILE: &str = "target/fuufu-ijou-v8-v9-en.tar.zstd";

    fn normal(builder: PackageBuilder) {
        let start = Instant::now();
        let mut output_file = File::create(NORMAL_FILE).unwrap();
        let mut output_file_buf_writer = BufWriter::new(&mut output_file);
        let _ = builder.clone().build(&mut output_file_buf_writer).unwrap();
        output_file_buf_writer.flush().unwrap();
        let build_time = Instant::now() - start;
        println!("Build Time Normal: {} s", build_time.as_secs_f64());
    }

    pub const ZSTD_IMAGES_FILE: &str = "target/fuufu-ijou-v8-v9-en-zstd-images.tar.zstd";

    fn zstd_images(builder: PackageBuilder) {
        let start = Instant::now();
        let mut output_file = File::create(ZSTD_IMAGES_FILE).unwrap();
        let mut output_file_buf_writer = BufWriter::new(&mut output_file);
        let _ = {
            let mut b = builder.clone();
            b.zstd_compressed_images(true);
            b
        }
        .build(&mut output_file_buf_writer)
        .unwrap();
        output_file_buf_writer.flush().unwrap();
        let build_time = Instant::now() - start;
        println!(
            "Build Time Zstd compressed images: {} s",
            build_time.as_secs_f64()
        );
    }

    pub const ZSTD_METADATA_FILE: &str = "target/fuufu-ijou-v8-v9-en-zstd-metadata.tar.zstd";

    fn zstd_metadata(builder: PackageBuilder) {
        let start = Instant::now();
        let mut output_file = File::create(ZSTD_METADATA_FILE).unwrap();
        let mut output_file_buf_writer = BufWriter::new(&mut output_file);
        let _ = {
            let mut b = builder.clone();
            b.zstd_compressed_images(true);
            b
        }
        .build(&mut output_file_buf_writer)
        .unwrap();
        output_file_buf_writer.flush().unwrap();
        let build_time = Instant::now() - start;
        println!(
            "Build Time Zstd compressed metadata: {} s",
            build_time.as_secs_f64()
        );
    }

    pub const ZSTD_ALL_FILE: &str = "target/fuufu-ijou-v8-v9-en-zstd-all.tar.zstd";

    fn zstd_all(builder: PackageBuilder) {
        let start = Instant::now();
        let mut output_file = File::create(ZSTD_ALL_FILE).unwrap();
        let mut output_file_buf_writer = BufWriter::new(&mut output_file);
        let _ = {
            let mut b = builder.clone();
            b.zstd_compressed_images(true);
            b.zstd_compressed_metadata(true);
            b
        }
        .build(&mut output_file_buf_writer)
        .unwrap();
        output_file_buf_writer.flush().unwrap();
        let build_time = Instant::now() - start;
        println!(
            "Build Time Zstd compressed all: {} s",
            build_time.as_secs_f64()
        );
    }
    pub fn main(builder: &PackageBuilder) {
        dict(builder);
        let bn = builder.clone();
        let bzi = builder.clone();
        let bzm = builder.clone();
        let bza = builder.clone();
        let n = thread::spawn(move || normal(bn));
        let zi = thread::spawn(move || zstd_images(bzi));
        let zm = thread::spawn(move || zstd_metadata(bzm));
        let za = thread::spawn(move || zstd_all(bza));
        n.join().unwrap();
        zi.join().unwrap();
        zm.join().unwrap();
        za.join().unwrap();
    }
}

mod archive {
    use std::{io::BufReader, path::Path};

    use api_core::data_push::Push;
    use emdx::Archive;

    use super::*;

    fn read_file<P: AsRef<Path>>(path: P, builder: &PackageBuilder) -> File {
        let initial_contents = builder.get_package_contents();
        let mut file = File::open(path).unwrap();
        {
            let mut archive = Archive::from_reader(BufReader::new(&mut file)).unwrap();
            {
                let manga_pull = archive.manga_pull(true).unwrap();
                assert_eq!(manga_pull.flatten().count(), initial_contents.data.len());
            }
            {
                let cover_pull = archive.cover_pull(true).unwrap();
                assert_eq!(
                    cover_pull.flatten().count(),
                    initial_contents
                        .data
                        .values()
                        .fold(0, |acc, manga| { acc + manga.covers.len() })
                );
            }
            {
                let chapter_pull = archive.chapter_pull(true).unwrap();
                assert_eq!(
                    chapter_pull.flatten().count(),
                    initial_contents
                        .data
                        .values()
                        .fold(0, |acc, manga| { acc + manga.chapters.len() })
                );
            }
            {
                let any_pull = archive.any_pull(true).unwrap();
                for data in any_pull.flatten() {
                    match data {
                        emdx::archive::pull::any::PossibleEntryData::Manga(d) => {
                            println!("Manga => {}", d.id);
                        }
                        emdx::archive::pull::any::PossibleEntryData::Chapter(d) => {
                            println!("Chapter => {}", d.id);
                        }
                        emdx::archive::pull::any::PossibleEntryData::Cover(d) => {
                            println!("Cover => {}", d.id);
                        }
                        emdx::archive::pull::any::PossibleEntryData::CoverImage {
                            filename,
                            file: _,
                        } => {
                            println!("CoverImage => {filename}");
                        }
                        emdx::archive::pull::any::PossibleEntryData::ChapterImage {
                            filename,
                            file: _,
                            chapter,
                            mode,
                        } => {
                            println!("ChapterImage({chapter} - {mode:?}) => {filename}");
                        }
                        emdx::archive::pull::any::PossibleEntryData::Any { tar_path, file: _ } => {
                            println!("Any => {tar_path:?}");
                        }
                    }
                }
            }
        }
        file
    }
    fn push_archive<P: AsRef<Path>>(path: P) -> File {
        let mut dir_options = DirsOptions::new_from_data_dir(format!(
            "{}.dir-options",
            path.as_ref().to_str().unwrap()
        ));
        let _ = dir_options.verify_and_init();
        let mut file = File::open(&path).unwrap();
        {
            let archive = Archive::from_reader(&mut file).unwrap();
            dir_options.push(archive).unwrap();
        }
        file
    }
    fn normal(builder: &PackageBuilder) {
        {
            let start = Instant::now();
            read_file(package::NORMAL_FILE, builder);
            let pull_time = Instant::now() - start;
            println!("normal bench Time: {} ms", pull_time.as_millis());
        }
        {
            let start = Instant::now();
            push_archive(package::NORMAL_FILE);
            let pull_time = Instant::now() - start;
            println!("normal push Time: {} ms", pull_time.as_millis());
        }
    }
    fn zstd_metadata(builder: &PackageBuilder) {
        {
            let start = Instant::now();
            read_file(package::ZSTD_METADATA_FILE, builder);
            let pull_time = Instant::now() - start;
            println!("zstd metadata bench Time: {} ms", pull_time.as_millis());
        }
        {
            let start = Instant::now();
            push_archive(package::ZSTD_METADATA_FILE);
            let pull_time = Instant::now() - start;
            println!("zstd metadata push Time: {} ms", pull_time.as_millis());
        }
    }
    fn zstd_images(builder: &PackageBuilder) {
        {
            let start = Instant::now();
            read_file(package::ZSTD_IMAGES_FILE, builder);
            let pull_time = Instant::now() - start;
            println!("zstd images bench Time: {} ms", pull_time.as_millis());
        }
        let start = Instant::now();
        push_archive(package::ZSTD_IMAGES_FILE);
        let pull_time = Instant::now() - start;
        println!("zstd images push Time: {} ms", pull_time.as_millis());
    }
    fn zstd_all(builder: &PackageBuilder) {
        {
            let start = Instant::now();
            read_file(package::ZSTD_ALL_FILE, builder);
            let pull_time = Instant::now() - start;
            println!("zstd metadata bench Time: {} ms", pull_time.as_millis());
        }
        {
            let start = Instant::now();
            push_archive(package::ZSTD_ALL_FILE);
            let pull_time = Instant::now() - start;
            println!("zstd metadata push Time: {} ms", pull_time.as_millis());
        }
    }
    pub fn main(builder: &PackageBuilder) {
        normal(builder);
        zstd_metadata(builder);
        zstd_images(builder);
        zstd_all(builder);
    }
}

fn main() {
    let start = Instant::now();
    let options = DirsOptions::new_from_data_dir("data");
    let mut builder = PackageBuilder::new(options.clone()).set_compress_image_to_jpeg(true);
    let chapters = {
        let manga = options
            .pull_all_mangas()
            .unwrap()
            .flatten()
            .to_filtered(MangaListDataPullFilterParams {
                title: Some("Fuufu Ijou".into()),
                ..Default::default()
            })
            .next()
            .unwrap();
        options
            .pull_all_chapter()
            .unwrap()
            .flatten()
            .to_filtered(ChapterListDataPullFilterParams {
                volumes: vec!["8".into(), "9".into()],
                translated_languages: vec![Language::English],
                manga_ids: vec![manga.id],
                ..Default::default()
            })
            .map(|chap| chap.id)
            .collect::<Vec<_>>()
    };
    let pull_time = Instant::now() - start;
    println!("Pulling Time: {} ms", pull_time.as_millis());
    let start = Instant::now();
    for chapter_id in chapters {
        builder.add_chapter(chapter_id, Default::default()).unwrap();
    }
    builder.set_compression_level(3);
    let add_time = Instant::now() - start;
    println!("Adding Time: {} ms", add_time.as_millis());
    package::main(&builder);
    archive::main(&builder);
    println!("Done!");
}
