use std::{fs::File, io::{ErrorKind, Write}, path::Path};
use async_stream::stream;
use tokio_stream::{Stream, StreamExt};
use log::info;
use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::{ApiObject, ApiData, v5::ChapterAttributes};
use mangadex_api_types_rust::RelationshipType;

use crate::{settings::{files_dirs::DirsOptions, file_history::HistoryEntry}, download::manga::download_manga, download::cover::cover_download_by_manga_id, core::{ManagerCoreResult, Error}, methods::get::GetChapterQuery, server::traits::AccessHistory};


use super::{manga::MangaUtils, collection::Collection, cover::CoverUtils};

#[derive(Clone)]
pub struct ChapterUtils{
    pub(crate) dirs_options : DirsOptions,
    pub(crate) http_client_ref : HttpClientRef
}

impl ChapterUtils {
    pub fn new(dirs_options : DirsOptions, http_client_ref : HttpClientRef) -> Self {
        Self { dirs_options, http_client_ref }
    }
    pub fn is_chapter_manga_there(&self, chap_id: String) -> ManagerCoreResult<bool>{
        if !chap_id.is_empty() {
            let path = self.dirs_options.chapters_add(format!("{}/data.json", chap_id).as_str());
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
    pub async fn update_chap_by_id(&self, id: String) -> ManagerCoreResult<serde_json::Value> {
        let path = self.dirs_options.chapters_add(format!("{}/data.json", id).as_str());

            let http_client = self.http_client_ref.lock().await.client.clone();
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
    pub async fn patch_manga_by_chapter<H>(&self, chap_id: String, history : &H) -> ManagerCoreResult<serde_json::Value>
    where
        H : AccessHistory
    {
        let path = self.dirs_options.chapters_add(format!("{}/data.json", chap_id).as_str());
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
        history.insert_in_history(&history_entry).await?;
        history.commit_rel(history_entry.get_data_type()).await?;
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
        history.remove_in_history(&history_entry).await?;
        history.commit_rel(history_entry.get_data_type()).await?;
        Ok(jsons)
    }
    pub fn get_chapter_by_id<T>(&self, chap_id: T) -> ManagerCoreResult<ApiObject<ChapterAttributes>> 
    where
        T : ToString
    {
        let path = self.dirs_options.chapters_add(format!("{}/data.json", chap_id.to_string()).as_str());
        let data : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_reader(File::open(path)?)?;
        Ok(data.data)
    }
    pub fn get_chapters_by_stream_id<'a, T>(&'a self, mut chap_ids: T) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + 'a> 
    where 
        T : Stream<Item = String> + std::marker::Unpin + 'a
    {
        Ok(
            stream! {
                while let Some(id) = chap_ids.next().await {
                    if let Ok(data_) = self.get_chapter_by_id(id) {
                        yield data_;
                    }
                }
            }
        )
    }
    pub fn get_chapters_by_vec_id<'a>(&'a self, chap_ids: Vec<String>) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + 'a> {
        Ok(
            stream! {
                for id in chap_ids {
                    if let Ok(data_) = self.get_chapter_by_id(id) {
                        yield data_;
                    }
                }
            }
        )
    }
    pub fn get_all_chapter_without_history<'a>(&'a self) -> ManagerCoreResult<impl Stream<Item = String> + 'a> {
        let file_dirs = self.dirs_options.clone();
        let path = file_dirs.chapters_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = std::fs::read_dir(path.as_str())?;
            Ok(stream! {
                for files in list_dir.flatten() {
                    if let Some(data) = files.file_name().to_str() {
                        if Path::new(format!("{}/data.json", file_dirs.chapters_add(data)).as_str()).exists() {
                            yield data.to_string()
                        }
                    }
                }
            })
        }else {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't find the chapter directory",
            )))
        }
    }
    pub async fn get_all_chapter<'a, H>(&'a self, parameters : Option<GetAllChapter>, history : &'a H)-> ManagerCoreResult<impl Stream<Item = String> + 'a>
        where 
            H : AccessHistory
    {
        let file_dirs = self.dirs_options.clone();
        let mut all_chapters = Box::pin(self.get_all_chapter_without_history()?);
        let parameters = parameters.unwrap_or_default();
        let mut hist = history.get_history_w_file_by_rel_or_init(RelationshipType::Chapter).await?;
        Ok(async_stream::stream! {
            let chapter_history = hist.get_history();
            if !parameters.only_fails {
                while let Some(data) = all_chapters.next().await {
                    if !parameters.include_fails{
                        if !chapter_history.is_in(
                                match uuid::Uuid::parse_str(data.as_str()){
                                    Ok(o) => o,
                                    Err(_) => uuid::Uuid::NAMESPACE_DNS
                            }) 
                        {
                            yield data.to_string()
                        }
                    }else {
                        yield data.to_string()
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
    } 
    pub async fn get_all_downloaded_chapters<'a, H>(&'a self, parameters : Option<GetChapterQuery>, history : &'a H) -> ManagerCoreResult<Collection<String>> 
        where 
            H : AccessHistory
    {
    if let Some(param) = parameters{
        let stream = self.get_all_chapter(Some(GetAllChapter::from(param.clone())), history).await?;
        let collection: Collection<String> = Collection::from_async_stream(stream, param.clone().limit.unwrap_or(10), param.offset.unwrap_or(0)).await?;
        Ok(collection)
    }else{
        let stream = self.get_all_chapter(None, history).await?;
        let collection: Collection<String> = Collection::from_async_stream(stream, 10, 0).await?;
        Ok(collection)
    }
}
}

impl From<MangaUtils> for ChapterUtils {
    fn from(value: MangaUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a MangaUtils> for ChapterUtils {
    fn from(value: &'a MangaUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl From<CoverUtils> for ChapterUtils {
    fn from(value: CoverUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a CoverUtils> for ChapterUtils {
    fn from(value: &'a CoverUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
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