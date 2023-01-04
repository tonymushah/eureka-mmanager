use std::fs::File;
use std::io::{Write, ErrorKind};
use std::path::Path;
use anyhow::Ok;
use log::{info};
use mangadex_api_schema::v5::{
    ChapterAttributes
};
use mangadex_api_schema::{
    ApiData, 
    ApiObject
};
use mangadex_api_types::RelationshipType;

use crate::settings::files_dirs::DirsOptions;

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

pub async fn find_all_downloades_by_manga_id(manga_id: String) -> anyhow::Result<serde_json::Value> {
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
    Ok(serde_json::json!(vecs))
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
    let manga_id =  match chapter
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::Manga){
            None => {
                return Err(anyhow::Error::new(std::io::Error::new(ErrorKind::Other, format!("can't find manga in the chapter {}", chap_id).as_str())));
            },
            Some(data) => data
        }
        .id;
        let http_client = reqwest::Client::new();
        let resp = send_request(http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, manga_id.hyphenated())), 5).await?;
        let mut file = File::create(
            DirsOptions::new()?
                    .mangas_add(format!("{}.json", manga_id.hyphenated()).as_str())
                    .as_str())?;
        file.write_all(&(resp.bytes().await?))?;
    
    let jsons = serde_json::json!({
            "result" : "ok",
            "type" : "manga",
            "id" : manga_id.hyphenated()
        });
    info!("downloaded {}.json", manga_id.hyphenated());
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
