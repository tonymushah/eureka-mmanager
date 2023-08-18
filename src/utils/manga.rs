use async_stream::stream;
use futures::Stream;
use mangadex_api_schema_rust::v5::{ChapterAttributes, MangaAttributes};
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use std::cmp::Ordering;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use tokio_stream::StreamExt;

use crate::core::ManagerCoreResult;
use crate::settings::files_dirs::DirsOptions;

use super::chapter::{get_all_chapter, get_chapter_by_id, get_chapters_by_stream_id};
use super::collection::Collection;
use super::cover::{get_all_cover, is_cover_there, get_cover_data};

pub async fn is_chap_related_to_manga(chap_id: String, manga_id: String) -> ManagerCoreResult<bool> {
    let chapter: ApiObject<ChapterAttributes> = get_chapter_by_id(chap_id)?;
    let mut is = false;
    for relas in chapter.relationships {
        if relas.type_ == RelationshipType::Manga && relas.id.hyphenated().to_string() == manga_id {
            is = true;
        }
    }
    Ok(is)
}

pub async fn find_all_downloades_by_manga_id(
    manga_id: String,
) -> ManagerCoreResult<impl Stream<Item = String>> {
    let stream = get_all_chapter(None)?;
    let mut stream = Box::pin(stream);
    let output = stream! {
        while let Some(chap) = stream.next().await {
            if let Ok(d) = is_chap_related_to_manga(chap.clone(), manga_id.clone()).await {
                if d {
                    yield chap;
                }
            };
        }
    };
    Ok(output)
}

pub async fn find_and_delete_all_downloades_by_manga_id(
    manga_id: String,
) -> ManagerCoreResult<serde_json::Value> {
    let mut vecs: Vec<String> = Vec::new();
    let mut stream = Box::pin(get_all_chapter(None)?);
    while let Some(files) = stream.next().await {
        let to_use = files;
        let to_insert = to_use.clone();
        let to_remove = to_use.clone();
        if let Ok(d) = is_chap_related_to_manga(to_use, manga_id.clone()).await {
            if d {
                vecs.push(to_insert);
                std::fs::remove_dir_all(DirsOptions::new()?.chapters_add(to_remove.as_str()))?
            }
        }
    }
    Ok(serde_json::json!(vecs))
}

pub fn get_downloaded_cover_of_a_manga(
    manga_id: String,
) -> ManagerCoreResult<impl Stream<Item = String>> {
    let mut vecs = Box::pin(get_all_cover()?);
    Ok(stream! {
        while let Some(data) = vecs.next().await {
            let manga_id = manga_id.clone();
            let data = data.clone();
            let data_clone = data.clone();
            if let core::result::Result::Ok(result) = is_cover_related_to_manga(manga_id, data) {
                if result {
                    yield data_clone;
                }
            }
        }
    })
}

pub async fn get_downloaded_cover_of_a_manga_collection(
    manga_id: String,
    offset: usize,
    limit: usize,
) -> ManagerCoreResult<Collection<String>> {
    let mut downloaded_covers = Box::pin(get_downloaded_cover_of_a_manga(manga_id)?);
    Collection::from_async_stream(&mut downloaded_covers, limit, offset).await
}

pub fn is_manga_cover_there(manga_id: String) -> Result<bool, std::io::Error> {
    if !manga_id.is_empty() {
        let path = match DirsOptions::new() {
            core::result::Result::Ok(data) => data,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            }
        }
        .mangas_add(format!("{}.json", manga_id).as_str());
        if !Path::new(path.as_str()).exists() {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "this manga hasn't been downloaded",
            ))
        } else {
            let manga_data: ApiData<ApiObject<MangaAttributes>> =
                serde_json::from_reader(File::open(path)?)?;
            let cover_id: uuid::Uuid = match manga_data
                .data
                .relationships
                .iter()
                .find(|rel| rel.type_ == RelationshipType::CoverArt)
            {
                None => {
                    return core::result::Result::Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "this manga has no cover_art",
                    ))
                }
                Some(d) => d.id,
            };
            is_cover_there(cover_id.to_string())
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "the manga_id should'nt be empty",
        ))
    }
}

pub fn is_cover_related_to_manga(
    manga_id: String,
    cover_id: String,
) -> ManagerCoreResult<bool> {
    match get_cover_data(cover_id)?.data.relationships.iter().find(|rel| rel.type_ == RelationshipType::Manga && rel.id.to_string() == manga_id){
        None => Ok(false),
        Some(_) => {
            is_manga_there(manga_id)
        }
    }
}

pub fn get_manga_data_by_id(
    manga_id: String,
) -> ManagerCoreResult<ApiObject<MangaAttributes>> {
    let file_dirs = DirsOptions::new_()?;
    let path = file_dirs.mangas_add(format!("{}.json", manga_id).as_str());
    if Path::new(path.as_str()).exists() {
        let data: ApiData<ApiObject<MangaAttributes>> =
            serde_json::from_str(std::fs::read_to_string(path.as_str())?.as_str())?;
        Ok(data.data)
    } else {
        Err(crate::core::Error::Io(std::io::Error::new(
            ErrorKind::NotFound,
            format!("manga {} not found", manga_id),
        )))
    }
}

pub fn get_manga_data_by_ids<T>(
    mut manga_ids: T,
) -> ManagerCoreResult<impl Stream<Item = ApiObject<MangaAttributes>>>
where
    T: Stream<Item = String> + std::marker::Unpin,
{
    Ok(stream! {
        while let Some(id) = manga_ids.next().await{
            if let Ok(data) = get_manga_data_by_id(id) {
                yield data;
            }
        }
    })
}

pub fn get_manga_data_by_ids_old(
    manga_ids: Vec<String>,
) -> ManagerCoreResult<impl Stream<Item = ApiObject<MangaAttributes>>> {
    Ok(stream! {
        for id in manga_ids {
            if let Ok(data) = get_manga_data_by_id(id) {
                yield data;
            }
        }
    })
}

pub fn get_all_downloaded_manga() -> ManagerCoreResult<impl Stream<Item = String>,> {
    let file_dirs = DirsOptions::new()?;
    let path = file_dirs.mangas_add("");
    if Path::new(path.as_str()).exists() {
        let list_dir = (std::fs::read_dir(path.as_str()))?;
        Ok(stream! {
            for file_ in list_dir.flatten() {
                if let Some(data) = file_.file_name().to_str() {
                    yield data.to_string().replace(".json", "")
                }
            }
        })
    } else {
        Err(crate::core::Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "can't find the manga directory",
        )))
    }
}

pub async fn get_downloaded_manga(
    offset: usize,
    limit: usize,
    title__: Option<String>,
) -> ManagerCoreResult<Collection<String>> {
    let vecs = Box::pin(get_all_downloaded_manga()?);
    let manga_data = Box::pin(get_manga_data_by_ids(vecs)?);
    if let Some(title) = title__ {
        let mut data: Vec<String> = manga_data
            .filter(|data| {
                for title_ in data.attributes.title.values() {
                    if title_
                        .to_lowercase()
                        .contains(title.to_lowercase().as_str())
                    {
                        return true;
                    }
                }
                for entry in &data.attributes.alt_titles {
                    for title_ in entry.values() {
                        if title_
                            .to_lowercase()
                            .contains(title.to_lowercase().as_str())
                        {
                            return true;
                        }
                    }
                }
                false
            })
            .map(|d| d.id.to_string())
            .collect()
            .await;
        Collection::new(&mut data, limit, offset)
    } else {
        let mut data: Vec<String> = manga_data
            .map(|d| d.id.to_string())
            .collect()
            .await;
        Collection::new(&mut data, limit, offset)
    }
}

pub async fn get_downloaded_chapter_of_a_manga(
    manga_id: String,
    offset: usize,
    limit: usize,
) -> ManagerCoreResult<Collection<String>> {
    let all_downloaded = get_all_downloaded_chapter_data(manga_id).await?;
    let mut data = Box::pin(all_downloaded);
    let to_use: Collection<String> = Collection::from_async_stream(&mut data, limit, offset)
        .await?
        .convert_to(|d| d.id.to_string())?;
    Ok(to_use)
}

pub fn is_manga_there(manga_id: String) -> ManagerCoreResult<bool> {
    if !manga_id.is_empty() {
        let path = match DirsOptions::new() {
            core::result::Result::Ok(data) => data,
            Err(e) => {
                return ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                )))
            }
        }
        .mangas_add(format!("{}.json", manga_id).as_str());
        ManagerCoreResult::Ok(Path::new(path.as_str()).exists())
    } else {
        ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(
            std::io::ErrorKind::Other,
            "the manga_id should'nt be empty",
        )))
    }
}

pub async fn get_all_downloaded_chapter_data(
    manga_id: String,
) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>>> {
    let data_ = Box::pin(find_all_downloades_by_manga_id(manga_id).await?);
    let data = get_chapters_by_stream_id(data_)?;
            let data = Box::pin(data);
            let mut data_vec: Vec<ApiObject<ChapterAttributes>> = data.collect().await;
            data_vec.sort_by(|a, b| {
                let a = match a.attributes.chapter.clone() {
                    None => return Ordering::Equal,
                    Some(d) => d,
                };
                let b = match b.attributes.chapter.clone() {
                    None => return Ordering::Equal,
                    Some(d) => d,
                };
                let a_chp = match a.parse::<usize>() {
                    core::result::Result::Ok(d) => d,
                    Err(_) => return Ordering::Equal,
                };
                let b_chp = match b.parse::<usize>() {
                    core::result::Result::Ok(d) => d,
                    Err(_) => return Ordering::Equal,
                };
                a_chp.cmp(&b_chp)
            });
            core::result::Result::Ok(stream! {
                for chapter in data_vec {
                    yield chapter
                }
            })
        
}
