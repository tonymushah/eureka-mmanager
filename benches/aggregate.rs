use std::sync::Arc;

use criterion::{BenchmarkId, Criterion};
use mangadex_desktop_api2::{settings::files_dirs::DirsOptions, utils::manga::MangaUtils};

async fn aggregate(manga_id: uuid::Uuid) {
    let manga_utils = MangaUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
        .with_id(manga_id.to_string());
    serde_json::to_string(&(manga_utils.aggregate_manga_chapters().await.unwrap())).unwrap();
}

async fn aggregate_stream(manga_id: uuid::Uuid) {
    let manga_utils = MangaUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
        .with_id(manga_id.to_string());
    serde_json::to_string(
        &(manga_utils
            .aggregate_manga_chapters_async_friendly()
            .await
            .unwrap()),
    )
    .unwrap();
}

fn criterion_benchmark(c: &mut Criterion) {
    let runtime = tokio::runtime::Runtime::new().unwrap();
    let manga_id = uuid::Uuid::try_parse("1c8f0358-d663-4d60-8590-b5e82890a1e3").unwrap();
    c.bench_with_input(
        BenchmarkId::new("aggregate_stream", manga_id),
        &manga_id,
        |b, &s| {
            b.to_async(&runtime).iter_batched(
                || s,
                aggregate_stream,
                criterion::BatchSize::LargeInput,
            );
        },
    );
    c.bench_with_input(
        BenchmarkId::new("aggregate", manga_id),
        &manga_id,
        |b, &s| {
            b.to_async(&runtime)
                .iter_batched(|| s, aggregate, criterion::BatchSize::LargeInput);
        },
    );
}

fn main(){
    let mut c = Criterion::default().with_plots().configure_from_args();
    criterion_benchmark(&mut c);
}
