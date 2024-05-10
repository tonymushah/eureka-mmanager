use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use crate::data_pulls::{
    chapter::ChapterListDataPullFilterParams, IntoFiltered, IntoParamedFilteredStream,
};

use itertools::Itertools;
use mangadex_api_input_types::manga::aggregate::MangaAggregateParam;
use mangadex_api_schema_rust::v5::{
    manga_aggregate::{ChapterAggregate, VolumeAggregate},
    ChapterObject, MangaAggregate,
};
use mangadex_api_types_rust::ResultType;
use tokio_stream::{Stream, StreamExt};

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct AggregateNumber(String);

impl<S> From<S> for AggregateNumber
where
    S: Into<String>,
{
    fn from(value: S) -> Self {
        Self(value.into())
    }
}

impl Deref for AggregateNumber {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for AggregateNumber {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl TryFrom<AggregateNumber> for f32 {
    type Error = std::num::ParseFloatError;
    fn try_from(value: AggregateNumber) -> Result<Self, Self::Error> {
        value.parse::<f32>()
    }
}

impl TryFrom<&AggregateNumber> for f32 {
    type Error = std::num::ParseFloatError;
    fn try_from(value: &AggregateNumber) -> Result<Self, Self::Error> {
        value.parse::<f32>()
    }
}

impl PartialOrd for AggregateNumber {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for AggregateNumber {
    fn cmp(&self, other: &Self) -> Ordering {
        let a: Option<f32> = self.try_into().ok();
        let b: Option<f32> = other.try_into().ok();
        a.partial_cmp(&b).unwrap_or(self.0.cmp(other))
    }
}

pub trait AsyncIntoMangaAggreagate {
    fn aggregate(
        self,
        params: MangaAggregateParam,
    ) -> impl std::future::Future<Output = MangaAggregate> + Send;
}

pub trait IntoMangaAggreagate {
    fn aggregate(self, params: MangaAggregateParam) -> MangaAggregate;
}

type VolumeAggregateCollector = BTreeMap<AggregateNumber, ChapterAggregate>;

type AggregateCollector = BTreeMap<AggregateNumber, VolumeAggregateCollector>;

fn none() -> String {
    String::from("none")
}

fn insert_in_collector(collector: &mut AggregateCollector, chapter: ChapterObject) {
    match collector
        .entry(AggregateNumber(
            chapter.attributes.volume.clone().unwrap_or_else(none),
        ))
        .or_default()
        .entry(AggregateNumber(
            chapter.attributes.chapter.clone().unwrap_or_else(none),
        )) {
        std::collections::btree_map::Entry::Vacant(e) => {
            e.insert(ChapterAggregate {
                chapter: chapter.attributes.chapter.clone().unwrap_or_else(none),
                id: chapter.id,
                others: Default::default(),
                count: 1,
            });
        }
        std::collections::btree_map::Entry::Occupied(mut e) => {
            let agg = e.get_mut();
            agg.others.push(chapter.id);
            agg.count += 1;
        }
    };
}

fn collector_to_aggregate(collector: AggregateCollector) -> MangaAggregate {
    let volumes = collector
        .into_iter()
        .filter_map(|(volume, chapters)| -> Option<VolumeAggregate> {
            let v_count = chapters
                .values()
                .map(|c| c.count)
                .reduce(|acc, e| acc + e)?;
            Some(VolumeAggregate {
                volume: volume.0,
                count: v_count,
                chapters: chapters.into_values().collect_vec(),
            })
        })
        .collect_vec();
    MangaAggregate {
        result: ResultType::Ok,
        volumes,
    }
}

impl<I> IntoMangaAggreagate for I
where
    I: Iterator<Item = ChapterObject>,
{
    fn aggregate(self, params: MangaAggregateParam) -> MangaAggregate {
        let filtered = self.to_filtered(Into::<ChapterListDataPullFilterParams>::into(params));
        let mut collector = AggregateCollector::new();
        for chapter in filtered {
            insert_in_collector(&mut collector, chapter);
        }
        collector_to_aggregate(collector)
    }
}

impl<S> AsyncIntoMangaAggreagate for S
where
    S: Stream<Item = ChapterObject> + Send,
{
    async fn aggregate(self, params: MangaAggregateParam) -> MangaAggregate {
        let mut filtered = self.to_filtered(Into::<ChapterListDataPullFilterParams>::into(params));
        let mut collector = AggregateCollector::new();
        while let Some(chapter) = filtered.next().await {
            insert_in_collector(&mut collector, chapter);
        }
        collector_to_aggregate(collector)
    }
}
