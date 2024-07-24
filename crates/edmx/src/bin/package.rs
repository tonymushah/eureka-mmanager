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
use edmx::PackageBuilder;
use mangadex_api_types_rust::Language;

fn normal(builder: PackageBuilder) {
    let start = Instant::now();
    let mut output_file = File::create("target/fuufu-ijou-v8-v9-en.tar.zstd").unwrap();
    let mut output_file_buf_writer = BufWriter::new(&mut output_file);
    let _ = builder.clone().build(&mut output_file_buf_writer).unwrap();
    output_file_buf_writer.flush().unwrap();
    let build_time = Instant::now() - start;
    println!("Build Time Normal: {} s", build_time.as_secs_f64());
}
fn zstd_images(builder: PackageBuilder) {
    let start = Instant::now();
    let mut output_file = File::create("target/fuufu-ijou-v8-v9-en-zstd-images.tar.zstd").unwrap();
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

fn zstd_metadata(builder: PackageBuilder) {
    let start = Instant::now();
    let mut output_file =
        File::create("target/fuufu-ijou-v8-v9-en-zstd-metadata.tar.zstd").unwrap();
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

fn zstd_all(builder: PackageBuilder) {
    let start = Instant::now();
    let mut output_file = File::create("target/fuufu-ijou-v8-v9-en-zstd-all.tar.zstd").unwrap();
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

fn main() {
    let start = Instant::now();
    let options = DirsOptions::new_from_data_dir("data");
    let mut builder = PackageBuilder::new(options.clone());
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
    println!("Done!");
}
