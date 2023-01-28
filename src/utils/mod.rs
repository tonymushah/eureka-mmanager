
use std::collections::HashMap;
use std::fs::File;
use std::hash::Hash;
use std::io::{Write, ErrorKind};
use std::path::Path;
use anyhow::Ok;
use log::{info};
use mangadex_api_schema::v5::{
    ChapterAttributes, CoverAttributes, MangaAttributes
};
use mangadex_api_schema::{
    ApiData, 
    ApiObject
};
use mangadex_api_types::RelationshipType;

use crate::cover_download::cover_download_by_manga_id;
use crate::manga_download::download_manga;
use crate::settings::file_history::HistoryEntry;
use crate::settings::files_dirs::DirsOptions;
use crate::settings::{insert_in_history, commit_rel, remove_in_history};

use self::collection::Collection;
pub mod collection;

pub async fn update_chap_by_id(id: String) -> anyhow::Result<serde_json::Value> {
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add(format!("{}/data.json", id).as_str());

        let http_client = reqwest::Client::new();
        let get_chapter = http_client
            .get(
                format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", 
                    mangadex_api::constants::API_URL, 
                    id
                )
            )
            .send()
            .await?;
        
            let bytes_ = get_chapter.bytes()
            .await?;
        
            let mut chapter_data = File::create((path).as_str())?;

        chapter_data.write_all(&bytes_)?;
        
        let jsons = std::fs::read_to_string(path.as_str())?;
        
        Ok(serde_json::from_str(jsons.as_str())?)
}

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

pub async fn patch_manga_by_chapter(chap_id: String) -> anyhow::Result<serde_json::Value> {
    let path = DirsOptions::new()?.chapters_add(format!("{}/data.json", chap_id).as_str());
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
    let manga =  match chapter
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::Manga){
            None => {
                return Err(anyhow::Error::new(std::io::Error::new(ErrorKind::Other, format!("can't find manga in the chapter {}", chap_id).as_str())));
            },
            Some(data) => data
        };
    let manga_id = manga.id;
    let history_entry = HistoryEntry::new(manga_id, manga.type_);
    insert_in_history(&history_entry)?;
    commit_rel(history_entry.get_data_type())?;
        let http_client = reqwest::Client::new();
        download_manga(http_client, manga_id).await?;
        match is_manga_cover_there(manga_id.to_string()) {
            core::result::Result::Ok(getted) => {
                if getted == false{
                    cover_download_by_manga_id(manga_id.to_string().as_str()).await?;
                }
            }, 
            Err(_) => {
                cover_download_by_manga_id(manga_id.to_string().as_str()).await?;
            }
        }
    let jsons = serde_json::json!({
            "result" : "ok",
            "type" : "manga",
            "id" : manga_id.hyphenated()
        });
    info!("downloaded {}.json", manga_id.hyphenated());
    remove_in_history(&history_entry)?;
    commit_rel(history_entry.get_data_type())?;
    Ok(jsons)
}

pub async fn send_request(to_use_arg: reqwest::RequestBuilder, tries_limits: u16) -> Result<reqwest::Response, std::io::Error>{
    let mut tries = 0;
    let to_use = to_use_arg;
    //let mut to_return : reqwest::Response;
    while tries < tries_limits {
        let resp = match to_use.try_clone(){
            None => {
                return Err(std::io::Error::new(ErrorKind::Other, "can't clone the request"));
            },
            Some(data) => data
        }.send().await;
        match resp {
            Err(_) => {
                tries = tries + 1;
            },
            core::result::Result::Ok(data) => {
                return core::result::Result::Ok(data);
            }
        }
    }
    Err(std::io::Error::new(ErrorKind::Other, "All tries failed to applies your request"))
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

pub fn is_chapter_manga_there(chap_id: String) -> Result<bool, std::io::Error>{
    if chap_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.chapters_add(format!("{}/data.json", chap_id).as_str());
        let chap_data : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_reader(File::open(path)?)?;
        let manga_id : uuid::Uuid = match chap_data.data.relationships.iter().find(|rel| rel.type_ == RelationshipType::Manga){
            Some(data) => data.id,
            None => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, "Seems like your chapter has no manga related to him"));
            }
        };
        return is_manga_there(format!("{}", manga_id));
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "the chap_id should'nt be empty"));
    }
}

pub fn is_cover_image_there(cover_id : String) -> Result<bool, std::io::Error>{
    if cover_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.covers_add(format!("{}.json", cover_id).as_str());
        let cover_data : ApiData<ApiObject<CoverAttributes>> = serde_json::from_reader(File::open(path)?)?;
        let cover_file_name = cover_data.data.attributes.file_name;
        let cover_file_name_path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.covers_add(format!("images/{}", cover_file_name).as_str());
        if Path::new(cover_file_name_path.as_str()).exists() {
            std::io::Result::Ok(true)
        }else{
            std::io::Result::Ok(false)
        }
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "the cover_id should'nt be empty"));
    }
}

pub fn is_cover_there(cover_id : String) -> Result<bool, std::io::Error>{
    if cover_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.covers_add(format!("{}.json", cover_id).as_str());
        if Path::new(path.as_str()).exists() {
            return is_cover_image_there(cover_id);
        }else{
            std::io::Result::Ok(false)
        }
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "the cover_id should'nt be empty"));
    }
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

pub fn get_cover_data(cover_id : String) -> Result<ApiData<ApiObject<CoverAttributes>>, std::io::Error>{
    let cover_id_clone = cover_id.clone();
    match is_cover_there(cover_id) {
        core::result::Result::Ok(is_there) => {
            if is_there == true{
                let path = match DirsOptions::new(){
                    core::result::Result::Ok(data) => data,
                    Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                }.covers_add(format!("{}.json", cover_id_clone).as_str());
                let data : ApiData<ApiObject<CoverAttributes>> = serde_json::from_str(std::fs::read_to_string(path)?.as_str())?;
                core::result::Result::Ok(data)
            }else{
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Cover not found"))
            }
        },
        Err(error) => Err(error)
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

pub fn query_string_to_hash_map(to_use: &str) -> Result<HashMap<String, String>, std::io::Error>{
    let mut to_return : HashMap<String, String> = HashMap::new();
    let query_part: Vec<&str> = to_use.split('&').collect();
    for parts in query_part {
        let query_part_parsed = match parts.split_once("=") {
            None => continue,
            Some(value) => value
        };
        to_return.insert(query_part_parsed.0.to_string(), query_part_parsed.1.to_string());
    }
    std::io::Result::Ok(to_return)
}

pub fn get_query_hash_value_or_else<T>(to_use: &HashMap<T, T>, to_get: T, or_else: T) -> T
    where T : std::cmp::Eq,
    T : Hash,
    T : Clone
{
    match to_use.get(&to_get) {
        Some(data) => data.clone(),
        None => or_else
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