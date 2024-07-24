use std::{fs::File, io::BufWriter, time::Instant};

use api_core::{
    data_pulls::{
        chapter::ChapterListDataPullFilterParams, manga::MangaListDataPullFilterParams,
        IntoFiltered,
    },
    DirsOptions,
};
use edmx::PackageBuilder;
use mangadex_api_types_rust::Language;

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
    let start = Instant::now();
    for chapter_id in chapters {
        builder.add_chapter(chapter_id, Default::default()).unwrap();
    }
    let add_time = Instant::now() - start;
    let start = Instant::now();
    let mut output_file = File::create("target/fuufu-ijou-v8-v9-en.tar.zstd").unwrap();
    let _ = builder.build(BufWriter::new(&mut output_file)).unwrap();
    let build_time = Instant::now() - start;
    println!("Done!");
    println!("Pulling Time: {} ms", pull_time.as_millis());
    println!("Adding Time: {} ms", add_time.as_millis());
    println!("Build Time: {} ms", build_time.as_millis());
}
