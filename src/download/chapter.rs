use futures::StreamExt;
// Imports used for downloading the pages to a file.
// They are not used because we're just printing the raw bytes.
use log::{info, warn};
use mangadex_api::{v5::MangaDexClient, utils::{download::chapter::DownloadMode}, HttpClientRef};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

use crate::{
    settings::{
        self, file_history::HistoryEntry,
    },
    utils::{chapter::{is_chapter_manga_there, patch_manga_by_chapter}, send_request},
    r#static::history::{commit_rel, insert_in_history, remove_in_history}
};

/// puting chapter data in a json data
async fn verify_chapter_and_manga(chapter_id: &Uuid, client: HttpClientRef, chapter_top_dir : &String) -> anyhow::Result<()>{
    let http_client = client.lock().await.client.clone();
    if !Path::new(format!("{}/data.json", chapter_top_dir).as_str()).exists() {
        let get_chapter = send_request(http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated())), 5).await?;
        if get_chapter.status().is_client_error() || get_chapter.status().is_server_error() {
            return anyhow::Result::Err(anyhow::Error::msg(format!("can't download the chapter {} data", chapter_id)));
        }
        let bytes_ = get_chapter.bytes().await?;
        let mut chapter_data = File::create(format!("{}/data.json", chapter_top_dir))?;
        chapter_data.write_all(&bytes_)?;
        info!("created data.json");
    }
    match is_chapter_manga_there(format!("{}", chapter_id)) {
        Ok(data) => {
            if !data {
                patch_manga_by_chapter(format!("{}", chapter_id), client).await?;
            }
        }
        Err(e) => {
            let error = e.to_string();
            warn!("Warning {}!", error);
            patch_manga_by_chapter(format!("{}", chapter_id), client).await?;
        }
    }
    anyhow::Ok(())
}

pub async fn download_chapter(chapter_id: &str, client_: HttpClientRef) -> anyhow::Result<serde_json::Value> {
    
    let chapter_id = Uuid::parse_str(chapter_id)?;
    let history_entry =
        HistoryEntry::new(chapter_id, mangadex_api_types_rust::RelationshipType::Chapter);
    match insert_in_history(&history_entry) {
        Ok(_) => (),
        Err(error) => {
            if error.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(anyhow::Error::new(error));
            }
        }
    };
    commit_rel(history_entry.get_data_type())?;
    
    let client = MangaDexClient::new_with_http_client_ref(client_.clone());
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let chapter_top_dir = files_dirs.chapters_add(chapter_id.hyphenated().to_string().as_str());
    let chapter_dir = format!("{}/data", chapter_top_dir);
    std::fs::create_dir_all(&chapter_dir)?;
    info!("chapter dir created");
    
    verify_chapter_and_manga(&chapter_id, client_, &chapter_top_dir).await?;
    
    let mut files_: Vec<String> = Vec::new();

    let stream = client.download().chapter(chapter_id).report(true).build()?.download_stream_with_checker(|filename, response| {
        let pre_file = match File::open(format!("{}/{}", chapter_dir.clone(), filename.filename.clone())){
            Ok(d) => d,
            Err(_) => return false
        };
        let content_length = match response.content_length() {
            None => return false,
            Some(ctt_lgth) => ctt_lgth
        };
        let pre_file_metadata = match pre_file.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return false
        };
        content_length == pre_file_metadata.len() 
    }).await?;
    let mut has_error = false;
    tokio::pin!(stream);
    let mut errors: Vec<String> = Vec::new();
    while let Some((result, index, len)) = stream.next().await {
        info!("{} - {}", index, len);
        match result {
            Ok((filename, bytes_)) => {
                if let Some(bytes) = bytes_ {
                    match File::create(format!("{}/{}", chapter_dir.clone(), filename.clone())){
                        Ok(mut file) => match file.write_all(&bytes) {
                            Ok(_) => {
                                info!("Downloaded {filename}");
                                files_.push(filename);
                            },
                            Err(e) => {
                                log::error!("{}", e.to_string());
                                errors.push(filename);
                            }
                        },
                        Err(e) => {
                            log::error!("{}", e.to_string());
                            errors.push(filename);
                        }
                    }
                }else {
                    info!("Skipped {}", filename);
                }
            },
            Err(error) => {
                log::error!("{}", error.to_string());
                has_error = true;
            },
        }
    }
    if !errors.is_empty() {
        has_error = true;
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "downloaded" : files_,
        "errors" : errors
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
    if !has_error {
        remove_in_history(&history_entry)?;
        commit_rel(history_entry.get_data_type())?;
    }
    
    Ok(jsons)
}
pub async fn download_chapter_saver(chapter_id: &str, client_: HttpClientRef) -> anyhow::Result<serde_json::Value> {
    let chapter_id = Uuid::parse_str(chapter_id)?;
    let history_entry =
        HistoryEntry::new(chapter_id, mangadex_api_types_rust::RelationshipType::Chapter);
    match insert_in_history(&history_entry) {
        Ok(_) => (),
        Err(error) => {
            if error.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(anyhow::Error::new(error));
            }
        }
    };
    commit_rel(history_entry.get_data_type())?;

    let client = MangaDexClient::new_with_http_client_ref(client_.clone());
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let chapter_top_dir = files_dirs.chapters_add(chapter_id.hyphenated().to_string().as_str());
    let chapter_dir = format!("{}/data-saver", chapter_top_dir);
    std::fs::create_dir_all(format!("{}/data-saver", chapter_top_dir))?;
    info!("chapter dir created");
    let mut files_: Vec<String> = Vec::new();

    let stream = client.download().chapter(chapter_id).report(true).mode(DownloadMode::DataSaver).build()?.download_stream_with_checker(|filename, response| {
        let pre_file = match File::open(format!("{}/{}", chapter_dir.clone(), filename.filename.clone())){
            Ok(d) => d,
            Err(_) => return false
        };
        let content_length = match response.content_length() {
            None => return false,
            Some(ctt_lgth) => ctt_lgth
        };
        let pre_file_metadata = match pre_file.metadata() {
            Ok(metadata) => metadata,
            Err(_) => return false
        };
        content_length == pre_file_metadata.len() 
    }).await?;
    let mut has_error = false;
    tokio::pin!(stream);
    let mut errors: Vec<String> = Vec::new();
    while let Some((result, index, len)) = stream.next().await {
        info!("{} - {}", index, len);
        match result {
            Ok((filename, bytes_)) => {
                if let Some(bytes) = bytes_ {
                    match File::create(format!("{}/{}", chapter_dir.clone(), filename.clone())){
                        Ok(mut file) => match file.write_all(&bytes) {
                            Ok(_) => {
                                info!("Downloaded {filename}");
                                files_.push(filename);
                            },
                            Err(e) => {
                                log::error!("{}", e.to_string());
                                errors.push(filename);
                            }
                        },
                        Err(e) => {
                            log::error!("{}", e.to_string());
                            errors.push(filename);
                        }
                    }
                }else {
                    info!("Skipped {}", filename);
                }
            },
            Err(error) => {
                log::error!("{}", error.to_string());
                has_error = true;
            },
        }
    }
    if !errors.is_empty() {
        has_error = true;
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "downloaded" : files_,
        "errors" : errors
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
    if !has_error {
        remove_in_history(&history_entry)?;
        commit_rel(history_entry.get_data_type())?;
    }
    
    Ok(jsons)
}

#[cfg(test)]
mod tests{
    use crate::r#static::history::init_static_history;

    use super::*;
    
    /// this will test the downloading for this chapter 
    /// https://mangadex.org/chapter/b8e7925e-581a-4c06-a964-0d822053391a
    /// 
    /// Dev note : Don't go there it's an H...
    #[tokio::test]
    async fn test_download_chapter_normal(){
        init_static_history().unwrap();
        let chapter_id = "b8e7925e-581a-4c06-a964-0d822053391a";
        let client = MangaDexClient::default();
        download_chapter(chapter_id, client.get_http_client()).await.unwrap();
    }
}