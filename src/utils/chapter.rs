use std::{fs::File, io::{ErrorKind, Write}};

use log::info;
use mangadex_api_schema::{ApiObject, ApiData, v5::ChapterAttributes};

use crate::{settings::{files_dirs::DirsOptions, file_history::HistoryEntry}, utils::manga::is_manga_cover_there, download::manga::download_manga, download::cover::cover_download_by_manga_id};

use crate::r#static::history::{insert_in_history, commit_rel, remove_in_history};

use super::manga::is_manga_there;

pub fn is_chapter_manga_there(chap_id: String) -> Result<bool, std::io::Error>{
    if chap_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.chapters_add(format!("{}/data.json", chap_id).as_str());
        let chap_data : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_reader(File::open(path)?)?;
        let manga_id : uuid::Uuid = match chap_data.data.relationships.iter().find(|rel| rel.type_ == mangadex_api::types::RelationshipType::Manga){
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
        .find(|related| related.type_ == mangadex_api::types::RelationshipType::Manga){
            None => {
                return Err(anyhow::Error::new(std::io::Error::new(ErrorKind::Other, format!("can't find manga in the chapter {}", chap_id).as_str())));
            },
            Some(data) => data
        };
    let manga_id = manga.id;
    let type_ = match manga.type_ {
        mangadex_api::types::RelationshipType::Manga => mangadex_api_types::RelationshipType::Manga,
        mangadex_api::types::RelationshipType::Chapter => mangadex_api_types::RelationshipType::Chapter,
        mangadex_api::types::RelationshipType::CoverArt => mangadex_api_types::RelationshipType::CoverArt,
        mangadex_api::types::RelationshipType::Author => mangadex_api_types::RelationshipType::Author,
        mangadex_api::types::RelationshipType::Artist => mangadex_api_types::RelationshipType::Artist,
        mangadex_api::types::RelationshipType::ScanlationGroup => mangadex_api_types::RelationshipType::ScanlationGroup,
        mangadex_api::types::RelationshipType::Tag => mangadex_api_types::RelationshipType::Tag,
        mangadex_api::types::RelationshipType::User => mangadex_api_types::RelationshipType::User,
        mangadex_api::types::RelationshipType::CustomList => mangadex_api_types::RelationshipType::CustomList,
        mangadex_api::types::RelationshipType::MappingId => mangadex_api_types::RelationshipType::MappingId,
        mangadex_api::types::RelationshipType::Member => mangadex_api_types::RelationshipType::Member,
        mangadex_api::types::RelationshipType::ReportReason => mangadex_api_types::RelationshipType::ReportReason,
        mangadex_api::types::RelationshipType::UploadSession => mangadex_api_types::RelationshipType::UploadSession,
        mangadex_api::types::RelationshipType::UploadSessionFile => mangadex_api_types::RelationshipType::UploadSessionFile,
        mangadex_api::types::RelationshipType::Collection => mangadex_api_types::RelationshipType::Collection,
        mangadex_api::types::RelationshipType::MangaRelation => mangadex_api_types::RelationshipType::MangaRelation,
        mangadex_api::types::RelationshipType::Unknown => mangadex_api_types::RelationshipType::Unknown,
        _ => mangadex_api_types::RelationshipType::Unknown
    };
    let history_entry = HistoryEntry::new(manga_id, type_);
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

pub fn get_chapter_by_id<T>(chap_id: T) -> anyhow::Result<ApiObject<ChapterAttributes>> 
    where
        T : ToString
{
    let file_dirs = DirsOptions::new()?;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data.json", chap_id.to_string()).as_str());
    let data : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_reader(File::open(path)?)?;
    anyhow::Ok(data.data)
}

pub fn get_chapters_by_vec_id(chap_ids: Vec<String>) -> anyhow::Result<Vec<ApiObject<ChapterAttributes>>> {
    let mut datas : Vec<ApiObject<ChapterAttributes>> = Vec::new();
    for id in chap_ids {
        datas.push(get_chapter_by_id(id)?);
    }
    anyhow::Ok(datas)
}

#[cfg(test)]
mod tests{
    use crate::utils::manga::find_all_downloades_by_manga_id;

    use super::*;
    #[tokio::test]
    pub async fn test_get_chapter_by_id(){
        let result = get_chapter_by_id("167fb8f3-1180-4b1c-ac02-a01dc24b8865".to_string());
        let data = result.unwrap();
        println!("{}", serde_json::to_string(&data).unwrap());
    }
    #[tokio::test]
    pub async fn test_get_chapters_by_vec_ids(){
        let manga_id = "17727b0f-c9f2-4ab5-a0b1-b7b0cf6c1fc8".to_string();
        let manga_downloads = find_all_downloades_by_manga_id(manga_id).await.unwrap();
        let datas = get_chapters_by_vec_id(manga_downloads).unwrap();
        for chap in datas {
            println!("{}", serde_json::to_string(&chap).unwrap())
        }
    }
}