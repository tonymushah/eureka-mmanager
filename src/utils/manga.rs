use mangadex_api_schema_rust::v5::{ChapterAttributes, MangaAttributes};
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use std::cmp::Ordering;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;

use crate::settings::files_dirs::DirsOptions;

use super::chapter::{get_all_chapter, get_chapters_by_vec_id, get_chapter_by_id};
use super::collection::Collection;
use super::cover::{get_all_cover, get_cover_data, is_cover_there};

pub async fn is_chap_related_to_manga(chap_id: String, manga_id: String) -> anyhow::Result<bool> {
    let chapter: ApiObject<ChapterAttributes> = get_chapter_by_id(chap_id)?;
    let mut is = false;
    for relas in chapter.relationships {
        if relas.type_ == RelationshipType::Manga && relas.id.hyphenated().to_string() == manga_id {
            is = true;
        }
    }
    Ok(is)
}

pub async fn find_all_downloades_by_manga_id(manga_id: String) -> anyhow::Result<Vec<String>> {
    let mut vecs: Vec<String> = Vec::new();
    for chap in get_all_chapter()? {
        match is_chap_related_to_manga(chap.clone(), manga_id.clone()).await {
            Ok(d) => {
                if d == true {
                    vecs.push(chap);
                }
            },
            Err(_) => ()
        };
    }
    Ok(vecs)
}

pub async fn find_and_delete_all_downloades_by_manga_id(
    manga_id: String,
) -> anyhow::Result<serde_json::Value> {
    let mut vecs: Vec<String> = Vec::new();
    for files in get_all_chapter()? {
        let to_use = files;
        let to_insert = to_use.clone();
        let to_remove = to_use.clone();
        if is_chap_related_to_manga(to_use, manga_id.clone()).await? == true {
            vecs.push(to_insert);
            std::fs::remove_dir_all(DirsOptions::new()?.chapters_add(to_remove.as_str()))?
        }
    }
    Ok(serde_json::json!(vecs))
}

pub fn get_downloaded_cover_of_a_manga(manga_id: String) -> Result<Vec<String>, std::io::Error> {
    let vecs: Vec<String> = get_all_cover()?;
    let mut related_covers: Vec<String> = Vec::new();
    vecs.iter().for_each(|data| {
        let manga_id = manga_id.clone();
        let data = data.clone();
        let data_clone = data.clone();
        match is_cover_related_to_manga(manga_id, data) {
            core::result::Result::Ok(result) => {
                if result == true {
                    related_covers.push(data_clone);
                }
            }
            Err(_) => (),
        }
    });
    return std::io::Result::Ok(related_covers);
}

pub fn get_downloaded_cover_of_a_manga_collection(
    manga_id: String,
    offset: usize,
    limit: usize,
) -> Result<Collection<String>, std::io::Error> {
    let mut downloaded_covers = get_downloaded_cover_of_a_manga(manga_id)?;
    core::result::Result::Ok(Collection::new(&mut downloaded_covers, limit, offset)?)
}

pub fn is_manga_cover_there(manga_id: String) -> Result<bool, std::io::Error> {
    if manga_id.is_empty() == false {
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
        if Path::new(path.as_str()).exists() == false {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "this manga hasn't been downloaded",
            ));
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
            return is_cover_there(cover_id.to_string());
        }
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "the manga_id should'nt be empty",
        ));
    }
}

pub fn is_cover_related_to_manga(
    manga_id: String,
    cover_id: String,
) -> Result<bool, std::io::Error> {
    let manga_id_clone = manga_id.clone();
    match is_manga_there(manga_id) {
        core::result::Result::Ok(is_manga_there_) => {
            if is_manga_there_ == true {
                let manga_id = manga_id_clone.clone();
                let manga_id = manga_id.as_str();
                let manga_id = match uuid::Uuid::parse_str(manga_id) {
                    core::result::Result::Ok(data) => data,
                    Err(error) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            error.to_string(),
                        ))
                    }
                };
                match is_cover_there(cover_id.to_string()) {
                    core::result::Result::Ok(is_there) => {
                        if is_there == true {
                            let data = get_cover_data(cover_id)?;
                            match data.data.relationships.iter().find(|rel| {
                                rel.type_ == RelationshipType::Manga && rel.id == manga_id
                            }) {
                                Some(_) => return core::result::Result::Ok(true),
                                None => core::result::Result::Ok(false),
                            }
                        } else {
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                "cover not found",
                            ));
                        }
                    }
                    Err(error) => {
                        return Err(error);
                    }
                }
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "manga not found",
                ))
            }
        }
        Err(error) => Err(error),
    }
}

pub fn get_manga_data_by_id(
    manga_id: String,
) -> Result<ApiObject<MangaAttributes>, std::io::Error> {
    let file_dirs = DirsOptions::new_()?;
    let path = file_dirs.mangas_add(format!("{}.json", manga_id).as_str());
    if Path::new(path.as_str()).exists() == true {
        let data: ApiData<ApiObject<MangaAttributes>> =
            serde_json::from_str(std::fs::read_to_string(path.as_str())?.as_str())?;
        Ok(data.data)
    } else {
        Err(std::io::Error::new(
            ErrorKind::NotFound,
            format!("manga {} not found", manga_id),
        ))
    }
}

pub fn get_manga_data_by_ids(
    manga_ids: Vec<String>,
) -> Result<Vec<ApiObject<MangaAttributes>>, std::io::Error> {
    let mut datas: Vec<ApiObject<MangaAttributes>> = Vec::new();
    for id in manga_ids {
        datas.push(get_manga_data_by_id(id)?);
    }
    Ok(datas)
}

pub fn get_all_downloaded_manga() -> Result<Vec<String>, std::io::Error> {
    let file_dirs = match DirsOptions::new() {
        core::result::Result::Ok(data) => data,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            ))
        }
    };
    let path = file_dirs.mangas_add("");
    if Path::new(path.as_str()).exists() == true {
        let list_dir = (std::fs::read_dir(path.as_str()))?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            vecs.push(
                match (files)?.file_name().to_str() {
                    Some(data) => data,
                    None => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "can't recongnize file",
                        ))
                    }
                }
                .to_string()
                .replace(".json", ""),
            );
        }
        return Ok(vecs);
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "can't find the manga directory",
        ));
    }
}

pub fn get_downloaded_manga(
    offset: usize,
    limit: usize,
    title__: Option<String>,
) -> Result<Collection<String>, std::io::Error> {
    let mut vecs = get_all_downloaded_manga()?;
    let manga_data = get_manga_data_by_ids(vecs.clone())?;
    match title__ {
        None => (),
        Some(title) => {
            vecs = manga_data
                .iter()
                .filter(|data| {
                    for (_, title_) in &data.attributes.title {
                        if title_
                            .to_lowercase()
                            .contains((&title).to_lowercase().as_str())
                        {
                            return true;
                        }
                    }
                    for entry in &data.attributes.alt_titles {
                        for (_, title_) in entry {
                            if title_
                                .to_lowercase()
                                .contains((&title).to_lowercase().as_str())
                            {
                                return true;
                            }
                        }
                    }
                    return false;
                })
                .map(|d| d.id.to_string())
                .collect();
        }
    }

    let collection: Collection<String> = Collection::new(&mut vecs, limit, offset)?;
    return std::io::Result::Ok(collection);
}

pub async fn get_downloaded_chapter_of_a_manga(
    manga_id: String,
    offset: usize,
    limit: usize,
) -> Result<Collection<String>, std::io::Error> {
    let all_downloaded = get_all_downloaded_chapter_data(manga_id).await;
    let mut data = match all_downloaded {
        core::result::Result::Ok(data) => data.iter().map(|d| d.id.to_string()).collect(),
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            ));
        }
    };
    let to_use: Collection<String> = Collection::new(&mut data, limit, offset)?;
    std::io::Result::Ok(to_use)
}

pub fn is_manga_there(manga_id: String) -> Result<bool, std::io::Error> {
    if manga_id.is_empty() == false {
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
        core::result::Result::Ok(Path::new(path.as_str()).exists())
    } else {
        return Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "the manga_id should'nt be empty",
        ));
    }
}

pub async fn get_all_downloaded_chapter_data(
    manga_id: String,
) -> Result<Vec<ApiObject<ChapterAttributes>>, std::io::Error> {
    let data = match find_all_downloades_by_manga_id(manga_id).await {
        anyhow::Result::Ok(d) => d,
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        }
    };
    match get_chapters_by_vec_id(data) {
        anyhow::Result::Ok(mut data) => {
            data.sort_by(|a, b| {
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
            core::result::Result::Ok(data)
        }
        Err(e) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                e.to_string(),
            ))
        }
    }
}
