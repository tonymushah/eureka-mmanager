use std::cmp::Ordering;
use std::fs::File;
use std::io::{ErrorKind};
use std::path::Path;
use anyhow::Ok;
use mangadex_api::types::RelationshipType;
use mangadex_api_schema::v5::{
    ChapterAttributes, MangaAttributes,
};
use mangadex_api_schema::{
    ApiData, 
    ApiObject
};

use crate::settings::files_dirs::DirsOptions;

use super::chapter::get_chapters_by_vec_id;
use super::collection::Collection;
use super::cover::{get_cover_data, is_cover_there};


pub async fn is_chap_related_to_manga(chap_id: String, manga_id: String) -> anyhow::Result<bool>{
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add(format!("{}/data.json", chap_id).as_str());
    let chapter : ApiData<ApiObject<ChapterAttributes>> = match serde_json::from_str(&(
            match std::fs::read_to_string(path.as_str()){
                core::result::Result::Ok(data) => data,
                Err(_) => {
                    return Err(anyhow::Error::new(std::io::Error::new(ErrorKind::Other, format!("can't find or read file {}", path).as_str())));
                }
            }
        )){
            core::result::Result::Ok(data) => data,
            Err(_) => {
                return Err(anyhow::Error::new(std::io::Error::new(ErrorKind::Other, "Can't covert to json")));
            }
        };
    let mut is = false;
    for relas in chapter.data.relationships{
        if relas.type_ == RelationshipType::Manga && relas.id.hyphenated().to_string() == manga_id{
            is = true;
        }
    }
    Ok(is)
}

pub async fn find_all_downloades_by_manga_id(manga_id: String) -> anyhow::Result<Vec<String>> {
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add("");
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let to_use = match files?.file_name().to_str(){
                None => {
                    return Err(anyhow::Error::new(std::io::Error::new(ErrorKind::Other, "Can't recognize file")))
                },
                Some(data) => data
            }.to_string();
            let to_insert = to_use.clone();
            if is_chap_related_to_manga(to_use, manga_id.clone()).await? == true {
                vecs.push(to_insert);
            }
        }
    Ok(vecs)
}

pub async fn find_and_delete_all_downloades_by_manga_id(manga_id: String) -> anyhow::Result<serde_json::Value> {
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add("");
        let list_dir = std::fs::read_dir(path.as_str())?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let to_use = match files?.file_name().to_str(){
                None => {
                    return Err(anyhow::Error::new(std::io::Error::new(ErrorKind::Other, "Can't recognize file")))
                },
                Some(data) => data
            }.to_string();
            let to_insert = to_use.clone();
            let to_remove = to_use.clone();
            if is_chap_related_to_manga(to_use, manga_id.clone()).await? == true {
                vecs.push(to_insert);
                std::fs::remove_dir_all(
                    DirsOptions::new()?
                        .chapters_add(to_remove.as_str())
                )?
            }
        }
    Ok(serde_json::json!(vecs))
}

pub fn get_all_downloaded_chapters(offset: usize, limit: usize) -> Result<Collection<String>, std::io::Error>{
    let file_dirs = match DirsOptions::new() {
        core::result::Result::Ok(data) => data,
        Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
    };
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add("");
    if Path::new(path.as_str()).exists() == true {
        let list_dir = (std::fs::read_dir(path.as_str()))?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            vecs.push(
                match 
                    (files)?.file_name().to_str()
                {
                    Some(data) => data,
                    None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "can't recongnize file"))
                }
                .to_string()
            );
        }
        let collection: Collection<String> = Collection::new(&mut vecs, limit, offset)?;
        return std::io::Result::Ok(collection);
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "can't find the manga directory"))
    }
}

pub fn get_downloaded_cover_of_a_manga(manga_id : String) -> Result<Vec<String>, std::io::Error>{
    let file_dirs = match DirsOptions::new() {
        core::result::Result::Ok(data) => data,
        Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
    };
    let path = file_dirs.covers_add("");
    if Path::new(path.as_str()).exists() == true {
        let list_dir = (std::fs::read_dir(path.as_str()))?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            match files {
                core::result::Result::Ok(file) => {
                    if match file.metadata(){
                        core::result::Result::Ok(data) => data,
                        Err(_) => continue
                    }.is_file() == true {
                        vecs.push(
                        match 
                            file.file_name().to_str()
                        {
                            Some(data) => data,
                            None => continue
                        }
                        .to_string()
                        .replace(".json", ""),
                        );
                    }
                },
                Err(_) => continue
            }
            
        }
        let mut related_covers : Vec<String> = Vec::new();
        vecs.iter().for_each(|data| {
            let manga_id = manga_id.clone();
            let data = data.clone();
            let data_clone = data.clone();
            match is_cover_related_to_manga(manga_id, data){
                core::result::Result::Ok(result) => {
                    if result == true{
                        related_covers.push(data_clone);
                    }
                },
                Err(_) => ()
            }
        });
        return std::io::Result::Ok(related_covers);
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "can't find the manga directory"))
    }
}

pub fn get_downloaded_cover_of_a_manga_collection(manga_id : String, offset: usize, limit: usize) -> Result<Collection<String>, std::io::Error>{
    let mut downloaded_covers = get_downloaded_cover_of_a_manga(manga_id)?;
    core::result::Result::Ok(Collection::new(&mut downloaded_covers, limit, offset)?)
}

pub fn is_manga_cover_there(manga_id : String) -> Result<bool, std::io::Error>{
    if manga_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.mangas_add(format!("{}.json", manga_id).as_str());
        if Path::new(path.as_str()).exists() == false {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "this manga hasn't been downloaded"));
        }else{
            let manga_data : ApiData<ApiObject<MangaAttributes>> = serde_json::from_reader(File::open(path)?)?;
            let cover_id : uuid::Uuid = match manga_data.data.relationships.iter().find(|rel| rel.type_ == RelationshipType::CoverArt){
                None => return core::result::Result::Err(std::io::Error::new(std::io::ErrorKind::Other, "this manga has no cover_art")),
                Some(d) => d.id
            };
            return is_cover_there(cover_id.to_string());
        }
        
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "the manga_id should'nt be empty"));
    }
}

pub fn is_cover_related_to_manga(manga_id : String, cover_id : String) -> Result<bool, std::io::Error>{
    let manga_id_clone = manga_id.clone();
    match is_manga_there(manga_id) {
        core::result::Result::Ok(is_manga_there_) =>{
            if is_manga_there_ == true {
                let manga_id = manga_id_clone.clone();
                let manga_id = manga_id.as_str();
                let manga_id = match uuid::Uuid::parse_str(manga_id) {
                    core::result::Result::Ok(data) => data,
                    Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
                };
                match is_cover_there(cover_id.to_string()) {
                    core::result::Result::Ok(is_there) => {
                        if is_there == true {
                            let data = get_cover_data(cover_id)?;
                            match data.data.relationships.iter().find(|rel| rel.type_ == RelationshipType::Manga && rel.id == manga_id) {
                                Some(_) => return core::result::Result::Ok(true),
                                None => core::result::Result::Ok(false)
                            }
                        }else{
                            return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "cover not found"))
                        }
                    },
                    Err(error)=> {
                        return Err(error);
                    }
                }
            }else{
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "manga not found"))
            }
        },
        Err(error) => Err(error)
    }
}

pub fn get_downloaded_manga(offset: usize, limit: usize)-> Result<Collection<String>, std::io::Error>{
    let file_dirs = match DirsOptions::new() {
        core::result::Result::Ok(data) => data,
        Err(error) => return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()))
    };
    let path = file_dirs.mangas_add("");
    if Path::new(path.as_str()).exists() == true {
        let list_dir = (std::fs::read_dir(path.as_str()))?;
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            vecs.push(
                match 
                    (files)?.file_name().to_str()
                {
                    Some(data) => data,
                    None => return Err(std::io::Error::new(std::io::ErrorKind::Other, "can't recongnize file"))
                }
                .to_string()
                .replace(".json", ""),
            );
        }
        let collection: Collection<String> = Collection::new(&mut vecs, limit, offset)?;
        return std::io::Result::Ok(collection);
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::NotFound, "can't find the manga directory"))
    }
    
}

pub async fn get_downloaded_chapter_of_a_manga(manga_id: String, offset: usize, limit: usize) -> Result<Collection<String>, std::io::Error> {
    let all_downloaded = find_all_downloades_by_manga_id(manga_id).await;
    let mut data = match all_downloaded {
        core::result::Result::Ok(data) => data,
        Err(error) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()));
        }
    };
    let to_use : Collection<String> = Collection::new(&mut data, limit, offset)?;
    std::io::Result::Ok(to_use)
}

pub fn is_manga_there(manga_id: String) -> Result<bool, std::io::Error>{
    if manga_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.mangas_add(format!("{}.json", manga_id).as_str());
        core::result::Result::Ok(Path::new(path.as_str()).exists())
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "the manga_id should'nt be empty"));
    }
}

pub async fn get_all_downloaded_chapter_data(manga_id : String) -> Result<Vec<ApiObject<ChapterAttributes>>, std::io::Error> {
    let data = match find_all_downloades_by_manga_id(manga_id).await {
        anyhow::Result::Ok(d) => d,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    };
    let mut data = match get_chapters_by_vec_id(data) {
        anyhow::Result::Ok(d) => d,
        Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
    };
    data.sort_by(|a, b| {
        let a = match a.attributes.chapter.clone() {
            None => return Ordering::Equal,
            Some(d) => d
        };
        let b = match b.attributes.chapter.clone() {
            None => return Ordering::Equal,
            Some(d) => d
        };
        let a_chp = match a.parse::<usize>() {
            core::result::Result::Ok(d) => d,
            Err(_) => return Ordering::Equal
        };
        let b_chp = match b.parse::<usize>() {
            core::result::Result::Ok(d) => d,
            Err(_) => return Ordering::Equal
        };
        a_chp.cmp(&b_chp)
    });
    core::result::Result::Ok(data)
}
