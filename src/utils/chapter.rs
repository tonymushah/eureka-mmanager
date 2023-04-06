use std::{fs::File, io::{ErrorKind, Write}};

use log::info;
use mangadex_api_schema::{ApiObject, ApiData, v5::ChapterAttributes};
use mangadex_api_types::RelationshipType;

use crate::{settings::{files_dirs::DirsOptions, file_history::HistoryEntry, insert_in_history, commit_rel, remove_in_history}, utils::manga::is_manga_cover_there, manga_download::download_manga, cover_download::cover_download_by_manga_id};

use super::manga::is_manga_there;

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
