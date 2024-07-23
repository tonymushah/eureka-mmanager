use std::{fs::File, io::BufWriter};

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
    for chapter_id in chapters {
        builder.add_chapter(chapter_id, Default::default()).unwrap();
    }
    let mut output_file = File::create("target/fuufu-ijou-v8-v9-en.tar.zstd").unwrap();
    builder.build(BufWriter::new(&mut output_file)).unwrap();
    println!("Done!");
}
