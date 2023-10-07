use std::sync::Arc;

use criterion::Criterion;
use mangadex_api_schema_rust::{v5::ChapterAttributes, ApiObject};
use mangadex_desktop_api2::{settings::files_dirs::DirsOptions, utils::chapter::ChapterUtils};
use tokio_stream::StreamExt;

async fn id_only() {
    ChapterUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
        .get_all_chapter_without_history()
        .unwrap()
        .collect::<Vec<String>>()
        .await;
}

async fn with_data() {
    ChapterUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
        .get_all_chapters_data()
        .unwrap()
        .collect::<Vec<ApiObject<ChapterAttributes>>>()
        .await;
}

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    c.bench_function("id_only", |b| {
        b.to_async(&runtime).iter(id_only);
    });
    c.bench_function("with_data", |b| {
        b.to_async(&runtime).iter(with_data);
    });
}

fn main() {
    let mut c = Criterion::default().with_plots().configure_from_args();
    criterion_benchmark(&mut c);
}
