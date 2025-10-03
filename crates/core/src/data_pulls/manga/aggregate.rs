use std::{
    cmp::Ordering,
    collections::BTreeMap,
    ops::{Deref, DerefMut},
};

use crate::data_pulls::{IntoFiltered, chapter::ChapterListDataPullFilterParams};

#[cfg(feature = "stream")]
use crate::data_pulls::IntoParamedFilteredStream;

use itertools::Itertools;
use mangadex_api_input_types::manga::aggregate::MangaAggregateParam;
use mangadex_api_schema_rust::v5::{
    ChapterObject, MangaAggregate,
    manga_aggregate::{ChapterAggregate, VolumeAggregate},
};
use mangadex_api_types_rust::ResultType;
#[cfg(feature = "stream")]
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
        let f_a: Option<f32> = self.try_into().ok();
        let f_b: Option<f32> = other.try_into().ok();
        match (f_a, f_b) {
            (None, None) => self.0.cmp(other),
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Greater,
            (Some(a), Some(b)) => a.partial_cmp(&b).unwrap_or(self.0.cmp(other)),
        }
    }
}

/// If you want to get [`MangaAggregate`] data from an [`Stream<Item = ChapterObject>`],
/// then this is great way to do that.
///
/// This is already implemented for every [`Stream<Item = ChapterObject>`] out there.
#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
pub trait AsyncIntoMangaAggreagate {
    fn aggregate(
        self,
        params: MangaAggregateParam,
    ) -> impl std::future::Future<Output = MangaAggregate> + Send;
}

/// If you want to get [`MangaAggregate`] data from an [`Iterator<Item = ChapterObject>`],
/// then this is great way to do that.
///
/// This is already implemented for every [`Iterator<Item = ChapterObject>`] out there.
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
            e.insert(non_exhaustive::non_exhaustive!(ChapterAggregate {
                chapter: chapter.attributes.chapter.clone().unwrap_or_else(none),
                id: chapter.id,
                others: Default::default(),
                count: 1,
            }));
        }
        std::collections::btree_map::Entry::Occupied(mut e) => {
            let agg = e.get_mut();
            agg.others.push(chapter.id);
            agg.count += 1;
        }
    };
}

trait ToMangaAgg {
    fn agg(self) -> MangaAggregate;
}

impl ToMangaAgg for AggregateCollector {
    fn agg(self) -> MangaAggregate {
        let volumes = self
            .into_iter()
            .filter_map(|(volume, chapters)| -> Option<VolumeAggregate> {
                let v_count = chapters
                    .values()
                    .map(|c| c.count)
                    .reduce(|acc, e| acc + e)?;
                Some(non_exhaustive::non_exhaustive!(VolumeAggregate {
                    volume: volume.0,
                    count: v_count,
                    chapters: chapters.into_values().collect_vec(),
                }))
            })
            .collect_vec();
        non_exhaustive::non_exhaustive!(MangaAggregate {
            result: ResultType::Ok,
            volumes: volumes
        })
    }
}

impl<I> IntoMangaAggreagate for I
where
    I: Iterator<Item = ChapterObject>,
{
    fn aggregate(self, params: MangaAggregateParam) -> MangaAggregate {
        self.to_filtered(Into::<ChapterListDataPullFilterParams>::into(params))
            .fold(AggregateCollector::new(), |mut collector, chapter| {
                insert_in_collector(&mut collector, chapter);
                collector
            })
            .agg()
    }
}
#[cfg(feature = "stream")]
#[cfg_attr(docsrs, doc(cfg(feature = "stream")))]
impl<S> AsyncIntoMangaAggreagate for S
where
    S: Stream<Item = ChapterObject> + Send,
{
    async fn aggregate(self, params: MangaAggregateParam) -> MangaAggregate {
        let filtered = self.to_filtered(Into::<ChapterListDataPullFilterParams>::into(params));
        filtered
            .fold(AggregateCollector::new(), |mut collector, chapter| {
                insert_in_collector(&mut collector, chapter);
                collector
            })
            .await
            .agg()
    }
}
