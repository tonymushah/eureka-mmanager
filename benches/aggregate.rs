use std::sync::Arc;

use criterion::{
    measurement::WallTime, AxisScale, BenchmarkGroup, BenchmarkId, Criterion, PlotConfiguration,
};
use mangadex_desktop_api2::{settings::files_dirs::DirsOptions, utils::manga::MangaUtils};

async fn aggregate(manga_id: uuid::Uuid) {
    let manga_utils = MangaUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
        .with_id(manga_id);
    serde_json::to_string(&(manga_utils.aggregate_manga_chapter_default().await.unwrap())).unwrap();
}

async fn aggregate_stream(manga_id: uuid::Uuid) {
    let manga_utils = MangaUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
        .with_id(manga_id);
    serde_json::to_string(&(manga_utils.aggregate_manga_chapter_default().await.unwrap())).unwrap();
}

fn criterion_benchmark(c: &mut BenchmarkGroup<'_, WallTime>) {
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

fn main() {
    let plot_config = PlotConfiguration::default().summary_scale(AxisScale::Logarithmic);
    let mut c = Criterion::default().with_plots().configure_from_args();
    let mut aggregate_ = c.benchmark_group("aggregate benchmark");
    aggregate_.plot_config(plot_config);
    criterion_benchmark(&mut aggregate_);
    aggregate_.finish();
}
