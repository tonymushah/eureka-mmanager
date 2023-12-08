use std::cmp::Ordering;

use mangadex_api_schema_rust::{
    v5::{
        manga_aggregate::{ChapterAggregate, VolumeAggregate},
        ChapterAttributes,
    },
    ApiObject,
};
use tokio_stream::{StreamExt, Stream};

use crate::settings::file_history::IsIn;

use super::{IsHere, ShouldBe};

impl IsIn<&ApiObject<ChapterAttributes>> for Vec<uuid::Uuid> {
    type Output = bool;

    fn is_in(&self, to_use: &ApiObject<ChapterAttributes>) -> Self::Output {
        self.iter().any(|d| *d == to_use.id)
    }
}

impl<'a> ShouldBe<'a, &'a ApiObject<ChapterAttributes>> for Vec<ChapterAggregate> {
    type Output = IsHere;

    fn should_be(
        self: &'a mut std::vec::Vec<mangadex_api_schema_rust::v5::manga_aggregate::ChapterAggregate>,
        to_use: &'a ApiObject<ChapterAttributes>,
    ) -> Self::Output {
        let attributes = &to_use.attributes;
        if let Some(should_be) = self
            .iter_mut()
            .find(|d| d.chapter == attributes.chapter.clone().unwrap_or(String::from("none")))
        {
            if should_be.id == to_use.id || should_be.others.is_in(to_use) {
                IsHere::AlreadyHere
            } else {
                should_be.others.push(to_use.id);
                should_be.count +=
                    <usize as TryInto<u32>>::try_into(should_be.others.len()).unwrap_or(0);
                IsHere::Inserted
            }
        } else {
            let new = ChapterAggregate {
                chapter: to_use
                    .attributes
                    .chapter
                    .clone()
                    .unwrap_or("none".to_string()),
                id: to_use.id,
                others: Vec::new(),
                count: 1,
            };
            self.push(new);
            IsHere::Inserted
        }
    }
}

impl<'a> ShouldBe<'a, &'a ApiObject<ChapterAttributes>> for VolumeAggregate {
    type Output = IsHere;

    fn should_be(
        self: &'a mut mangadex_api_schema_rust::v5::manga_aggregate::VolumeAggregate,
        to_use: &'a ApiObject<ChapterAttributes>,
    ) -> Self::Output {
        if let IsHere::Inserted = self.chapters.should_be(to_use) {
            let mut count = 0;
            self.chapters.iter().for_each(|data| {
                count += data.count;
            });
            self.count = count;
            IsHere::Inserted
        } else {
            IsHere::AlreadyHere
        }
    }
}

impl<'a> ShouldBe<'a, &'a ApiObject<ChapterAttributes>> for Vec<VolumeAggregate> {
    type Output = IsHere;

    fn should_be(&'a mut self, to_use: &'a ApiObject<ChapterAttributes>) -> Self::Output {
        let attributes = &to_use.attributes;
        if let Some(here) = self.iter_mut().find(|d| {
            d.volume
                .cmp(&attributes.volume.clone().unwrap_or("none".to_string()))
                == Ordering::Equal
        }) {
            here.should_be(to_use)
        } else {
            let data = VolumeAggregate {
                volume: attributes.volume.clone().unwrap_or("none".to_string()),
                count: 1,
                chapters: vec![ChapterAggregate {
                    chapter: attributes.chapter.clone().unwrap_or("none".to_string()),
                    id: to_use.id,
                    count: 1,
                    others: Default::default(),
                }],
            };
            self.push(data);
            IsHere::Inserted
        }
    }
}

pub async fn group_chapter_to_volume_aggregate<I>(mut input: I) -> Vec<VolumeAggregate>
where
    I: Stream<Item = ApiObject<ChapterAttributes>> + Unpin,
{
    let mut data: Vec<VolumeAggregate> = Vec::new();
    while let Some(chapter) = input.next().await {
        data.should_be(&chapter);
    }
    data.iter_mut().for_each(|data| {
        data.chapters.sort_by(|a, b| {
            let a = match a.chapter.parse::<f32>() {
                Ok(d) => d,
                Err(_) => return Ordering::Equal,
            };
            let b = match b.chapter.parse::<f32>() {
                Ok(d) => d,
                Err(_) => return Ordering::Equal,
            };
            a.total_cmp(&b)
        });
    });
    data.sort_by(|a, b| {
        let a = match a.volume.parse::<f32>() {
            Ok(d) => d,
            Err(_) => return Ordering::Equal,
        };
        let b = match b.volume.parse::<f32>() {
            Ok(d) => d,
            Err(_) => return Ordering::Equal,
        };
        a.total_cmp(&b)
    });
    data
}

/*pub fn chapter_vec_to_chapter_aggregate_vec(input : Vec<ApiObject<ChapterAttributes>>) -> Result<()> {
    ChapterAggregate{

    }
    Ok(())
}*/

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use crate::{settings::files_dirs::DirsOptions, utils::manga::MangaUtils};

    #[tokio::test]
    async fn test_to_volume_aggregate() {
        let manga_id = "1c8f0358-d663-4d60-8590-b5e82890a1e3".to_string();
        let manga_utils =
            MangaUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
                .with_id(manga_id);
        println!(
            "{}",
            serde_json::to_string(
                &(manga_utils
                    .aggregate_manga_chapters_async_friendly()
                    .await
                    .unwrap())
            )
            .unwrap()
        );
    }
}
