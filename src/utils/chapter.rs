use std::{fs::File, io::{ErrorKind, Write}, path::Path};
use async_stream::stream;
use tokio_stream::{Stream, StreamExt};
use log::info;
use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::{ApiObject, ApiData, v5::ChapterAttributes};
use mangadex_api_types_rust::RelationshipType;

use crate::{settings::{files_dirs::DirsOptions, file_history::HistoryEntry}, utils::manga::is_manga_cover_there, download::manga::download_manga, download::cover::cover_download_by_manga_id, core::{ManagerCoreResult, Error}, methods::get::GetChapterQuery, r#static::history::get_history_w_file_by_rel_or_init};

use crate::r#static::history::{insert_in_history, commit_rel, remove_in_history};

use super::{manga::is_manga_there, collection::Collection};

pub fn is_chapter_manga_there(chap_id: String) -> ManagerCoreResult<bool>{
    if !chap_id.is_empty() {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, e.to_string())))
        }.chapters_add(format!("{}/data.json", chap_id).as_str());
        let chap_data : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_reader(File::open(path)?)?;
        let manga_id : uuid::Uuid = match chap_data.data.relationships.iter().find(|rel| rel.type_ == RelationshipType::Manga){
            Some(data) => data.id,
            None => {
                return ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "Seems like your chapter has no manga related to him")));
            }
        };
        is_manga_there(format!("{}", manga_id))
    }else{
        ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(std::io::ErrorKind::Other, "the chap_id should'nt be empty")))
    }
}

pub async fn update_chap_by_id(id: String, client : HttpClientRef) -> ManagerCoreResult<serde_json::Value> {
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add(format!("{}/data.json", id).as_str());

        let http_client = client.lock().await.client.clone();
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

pub async fn patch_manga_by_chapter(chap_id: String, client : HttpClientRef) -> ManagerCoreResult<serde_json::Value> {
    let path = DirsOptions::new()?.chapters_add(format!("{}/data.json", chap_id).as_str());
    let chapter : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_str(&(std::fs::read_to_string(path.as_str()))?)?;
    let manga =  match chapter
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::Manga){
            None => {
                return Err(Error::Io(std::io::Error::new(ErrorKind::Other, format!("can't find manga in the chapter {}", chap_id).as_str())));
            },
            Some(data) => data
        };
    let manga_id = manga.id;
    let type_ = manga.type_;
    let history_entry = HistoryEntry::new(manga_id, type_);
    insert_in_history(&history_entry)?;
    commit_rel(history_entry.get_data_type())?;
        download_manga(client.clone(), manga_id).await?;
        match is_manga_cover_there(manga_id.to_string()) {
            core::result::Result::Ok(getted) => {
                if !getted{
                    cover_download_by_manga_id(manga_id.to_string().as_str(), client.clone()).await?;
                }
            }, 
            Err(_) => {
                cover_download_by_manga_id(manga_id.to_string().as_str(), client).await?;
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

pub fn get_chapter_by_id<T>(chap_id: T) -> ManagerCoreResult<ApiObject<ChapterAttributes>> 
    where
        T : ToString
{
    let file_dirs = DirsOptions::new()?;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add(format!("{}/data.json", chap_id.to_string()).as_str());
    let data : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_reader(File::open(path)?)?;
    Ok(data.data)
}

pub  fn get_chapters_by_stream_id<T>(mut chap_ids: T) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>>> 
    where T : Stream<Item = String> + std::marker::Unpin
{
    Ok(
        stream! {
            while let Some(id) = chap_ids.next().await {
                if let Ok(data_) = get_chapter_by_id(id) {
                    yield data_;
                }
            }
        }
    )
}

pub  fn get_chapters_by_vec_id(chap_ids: Vec<String>) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>>> {
    Ok(
        stream! {
            for id in chap_ids {
                if let Ok(data_) = get_chapter_by_id(id) {
                    yield data_;
                }
            }
        }
    )
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
        let manga_downloads = Box::pin(find_all_downloades_by_manga_id(manga_id).await.unwrap());
        let datas = get_chapters_by_stream_id(manga_downloads).unwrap();
        tokio::pin!(datas);
        while let Some(chap) = datas.next().await{
            println!("{}", serde_json::to_string(&chap).unwrap());
        }
    }
}

#[derive(Clone)]
pub struct GetAllChapter{
    pub include_fails : bool,
    pub only_fails : bool
}

impl Default for GetAllChapter{
    fn default() -> Self {
        Self { include_fails: true, only_fails : false }
    }
}

impl From<GetChapterQuery> for GetAllChapter{
    fn from(value: GetChapterQuery) -> Self {
        Self { include_fails: value.include_fails.unwrap_or(true), only_fails: value.only_fails.unwrap_or(false) }
    }
}

pub fn get_all_chapter(parameters : Option<GetAllChapter>)-> ManagerCoreResult<impl Stream<Item = String>>{
    let parameters = parameters.unwrap_or_default();
    let file_dirs = DirsOptions::new()?;
    //let file_dir_clone = file_dirs.clone();
    let path = file_dirs.chapters_add("");
    if Path::new(path.as_str()).exists() {
        let chapter_history = get_history_w_file_by_rel_or_init(RelationshipType::Chapter)?.get_history();
        let list_dir = std::fs::read_dir(path.as_str())?;
        Ok(async_stream::stream! {
            if !parameters.only_fails {
                for files in list_dir.flatten() {
                    if let Some(data) = files.file_name().to_str() {
                        if !parameters.include_fails{
                            if 
                                Path::new(format!("{}/data.json", file_dirs.chapters_add(data)).as_str()).exists() 
                                && 
                                chapter_history.is_in(
                                    match uuid::Uuid::parse_str(data){
                                        Ok(o) => o,
                                        Err(_) => uuid::Uuid::NAMESPACE_DNS
                                }) 
                            {
                                yield data.to_string()
                            }
                        }else if Path::new(format!("{}/data.json", file_dirs.chapters_add(data)).as_str()).exists() {
                            yield data.to_string()
                        }
                    }
                }
            }else{
                for entry in chapter_history.clone().into_iter(){
                    if Path::new(format!("{}/data.json", file_dirs.chapters_add(entry.to_string().as_str())).as_str()).exists() {
                        yield entry.to_string()
                    }
                }
            }
        })
    } else {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "can't find the chapter directory",
        )))
    }
}

pub async fn get_all_downloaded_chapters(parameters : Option<GetChapterQuery>) -> ManagerCoreResult<Collection<String>> {
    if let Some(param) = parameters{
        let stream = get_all_chapter(Some(GetAllChapter::from(param.clone())))?;
        let collection: Collection<String> = Collection::from_async_stream(stream, param.clone().limit.unwrap_or(10), param.offset.unwrap_or(0)).await?;
        Ok(collection)
    }else{
        let stream = get_all_chapter(None)?;
        let collection: Collection<String> = Collection::from_async_stream(stream, 10, 0).await?;
        Ok(collection)
    }
}
