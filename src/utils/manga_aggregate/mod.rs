use std::{cmp::Ordering, collections::HashMap, io::Result, vec};

use mangadex_api_schema_rust::{
    v5::{
        manga_aggregate::{ChapterAggregate, VolumeAggregate},
        ChapterAttributes,
    },
    ApiObject,
};

pub mod stream;

pub type ChapterHashMap = HashMap<String, Vec<ApiObject<ChapterAttributes>>>;

fn group_chapter_to_chapter_hash_map(input: Vec<ApiObject<ChapterAttributes>>) -> ChapterHashMap {
    let mut data: ChapterHashMap = ChapterHashMap::new();
    for chap in input {
        let chap_ = chap.clone();
        let volume = match chap.attributes.chapter {
            None => "none".to_string(),
            Some(d) => d,
        };
        match data.get_mut(&volume) {
            None => {
                data.insert(volume, vec![chap_]);
            }
            Some(arr) => {
                arr.push(chap_);
            }
        }
    }
    data
}

fn chap_hashmapentry_to_chapter_aggregate(
    input: (String, Vec<ApiObject<ChapterAttributes>>),
) -> Result<ChapterAggregate> {
    let id = match input.1.get(0) {
        None => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "The input vector is empty",
            ));
        }
        Some(d) => d.clone(),
    };
    let others: Vec<uuid::Uuid> = match input.1.get(1..(input.1.len())) {
        None => Vec::new(),
        Some(d) => d.iter().map(|d_| d_.id).collect(),
    };
    let chapter: ChapterAggregate = ChapterAggregate {
        chapter: input.0,
        id: id.id,
        others,
        count: input.1.len() as u32,
    };
    Ok(chapter)
}

fn group_chapter_to_chapter_aggregate(
    input: Vec<ApiObject<ChapterAttributes>>,
) -> Result<Vec<ChapterAggregate>> {
    let data = group_chapter_to_chapter_hash_map(input);
    let mut returns: Vec<ChapterAggregate> = Vec::new();
    for chunk in data {
        returns.push(chap_hashmapentry_to_chapter_aggregate(chunk)?);
    }
    returns.sort_by(|a, b| {
        let a_chp = match a.chapter.parse::<f32>() {
            Ok(d) => d,
            Err(_) => return Ordering::Equal,
        };
        let b_chp = match b.chapter.parse::<f32>() {
            Ok(d) => d,
            Err(_) => return Ordering::Equal,
        };
        a_chp.total_cmp(&b_chp)
    });
    Ok(returns)
}

fn chapter_volume_hashmap_entry_to_volume_aggregate(
    (volume, chapters): (String, Vec<ApiObject<ChapterAttributes>>),
) -> Result<VolumeAggregate> {
    let chapters = group_chapter_to_chapter_aggregate(chapters)?;
    Ok(VolumeAggregate {
        volume,
        count: chapters.len() as u32,
        chapters,
    })
}

pub type ChapterVolumeHashMap = HashMap<String, Vec<ApiObject<ChapterAttributes>>>;
/// Convert an array of chapter to an HashMap with
/// - Key : volume number
/// - Value : The chapter Attributes
fn group_chapter_to_volume_hash_map(
    input: Vec<ApiObject<ChapterAttributes>>,
) -> Result<ChapterVolumeHashMap> {
    let mut data: ChapterVolumeHashMap = ChapterVolumeHashMap::new();
    for chap in input {
        let chap_ = chap.clone();
        let volume = match chap.attributes.volume {
            None => "none".to_string(),
            Some(d) => d,
        };
        match data.get_mut(&volume) {
            None => {
                data.insert(volume, vec![chap_]);
            }
            Some(arr) => {
                arr.push(chap_);
            }
        }
    }
    Ok(data)
}

pub fn group_chapter_to_volume_aggregate(
    input: Vec<ApiObject<ChapterAttributes>>,
) -> Result<Vec<VolumeAggregate>> {
    let mut data: Vec<VolumeAggregate> = Vec::new();
    for in_ in group_chapter_to_volume_hash_map(input)? {
        data.push(chapter_volume_hashmap_entry_to_volume_aggregate(in_)?);
    }
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
    Ok(data)
}

/*pub fn chapter_vec_to_chapter_aggregate_vec(input : Vec<ApiObject<ChapterAttributes>>) -> Result<()> {
    ChapterAggregate{

    }
    Ok(())
}*/

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use tokio_stream::StreamExt;

    use crate::{settings::files_dirs::DirsOptions, utils::manga::MangaUtils};

    use super::*;

    #[tokio::test]
    async fn test_to_volume_hash_map() {
        let manga_id = "d58eb211-a1ae-426c-b504-fc88253de600".to_string();
        let manga_utils =
            MangaUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default());
        let data: Vec<ApiObject<ChapterAttributes>> = (manga_utils
            .with_id(manga_id)
            .get_all_downloaded_chapter_data()
            )
            .unwrap()
            .collect()
            .await;
        for (volume, chapters) in group_chapter_to_volume_hash_map(data).unwrap() {
            println!(
                "\"{}\" : {}",
                volume,
                serde_json::to_string(&(group_chapter_to_chapter_aggregate(chapters).unwrap()))
                    .unwrap()
            );
        }
    }
    #[tokio::test]
    async fn test_to_volume_aggregate() {
        let manga_id = "d58eb211-a1ae-426c-b504-fc88253de600".to_string();
        let manga_utils =
            MangaUtils::new(Arc::new(DirsOptions::new().unwrap()), Default::default())
                .with_id(manga_id);
        println!(
            "{}",
            serde_json::to_string(&(manga_utils.aggregate_manga_chapters().await.unwrap()))
                .unwrap()
        );
    }
}
