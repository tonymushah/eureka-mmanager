// Imports used for downloading the pages to a file.
// They are not used because we're just printing the raw bytes.
use log::{info, warn};
use mangadex_api::{v5::MangaDexClient, utils::get_reqwest_client, HttpClientRef};
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

use crate::{
    settings::{
        self, file_history::HistoryEntry,
    },
    utils::{self, chapter::{is_chapter_manga_there, patch_manga_by_chapter}, send_request},
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

async fn download_chapter_file<U>(
    http_client : &reqwest::Client,
    page_url : U,
    path_to_use : &String,
    files_ : &mut Vec<String>,
    filename : &String
) -> anyhow::Result<()>
where 
    U : reqwest::IntoUrl
{
    match utils::send_request(http_client.get(page_url), 5).await {
            Ok(res) => {
                match File::open(path_to_use) {
                    Ok(file) => {
                        let res_length = res.content_length();
                        // The data should be streamed rather than downloading the data all at once.
                        if !Path::new(path_to_use.as_str()).exists()
                            || res_length.is_none()
                            || file.metadata()?.len()
                                != match res_length {
                                    Some(d) => {
                                        //info!("data length : {}", d);
                                        d
                                    }
                                    None => 0,
                                }
                        {
                            match res.bytes().await {
                                core::result::Result::Err(error) => {
                                    //info!("error on fetching data : {}", error.to_string());
                                    return Err(anyhow::Error::new(error));
                                }
                                core::result::Result::Ok(bytes) => {
                                    let mut file = File::create(path_to_use)?;
                                    match file.write_all(&bytes) {
                                        core::result::Result::Err(error) => {
                                            //info!(" at file write_all : {}", error.to_string());
                                            return Err(anyhow::Error::new(error));
                                        }
                                        core::result::Result::Ok(_) => {
                                            info!("downloaded {} ", &filename);
                                            files_.push((&filename).to_string());
                                        }
                                    };
                                }
                            };
                            // This is where you would download the file but for this example,
                            // we're just printing the raw data.
                        }
                    }
                    Err(_) => {
                        match res.bytes().await {
                            core::result::Result::Err(error) => {
                                //info!("error on fetching data : {}", error.to_string());
                                return Err(anyhow::Error::new(error));
                            }
                            core::result::Result::Ok(bytes) => {
                                let mut file = File::create(path_to_use)?;
                                match file.write_all(&bytes) {
                                    core::result::Result::Err(error) => {
                                        //info!(" at file write_all : {}", error.to_string());
                                        return Err(anyhow::Error::new(error));
                                    }
                                    core::result::Result::Ok(_) => {
                                        info!("downloaded {} ", &filename);
                                        files_.push((&filename).to_string());
                                    }
                                };
                            }
                        };
                    }
                };
            }
            Err(error) => {
                return Err(anyhow::Error::new(error));
            }
        };
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
    let at_home = client
        .at_home()
        .server()
        .chapter_id(&chapter_id)
        .build()?
        .send()
        .await?;
    let http_client = get_reqwest_client(&client).await;
    verify_chapter_and_manga(&chapter_id, client_, &chapter_top_dir).await?;
    let mut files_: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data;
    for filename in page_filenames {
        let path_to_use = format!("{}/{}", chapter_dir, &filename);
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home.base_url.join(&format!(
            "/{quality_mode}/{chapter_hash}/{page_filename}",
            quality_mode = "data",
            chapter_hash = at_home.chapter.hash,
            page_filename = filename
        ))?;
        download_chapter_file(&http_client, page_url, &path_to_use, &mut files_, &filename).await?;
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "downloaded" : files_
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
    remove_in_history(&history_entry)?;
    commit_rel(history_entry.get_data_type())?;
    Ok(jsons)
}
pub async fn download_chapter_saver(chapter_id: &str, client_: HttpClientRef) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client_.clone());
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let chapter_id = Uuid::parse_str(chapter_id)?;
    let chapter_top_dir = files_dirs.chapters_add(chapter_id.hyphenated().to_string().as_str());
    let chapter_dir = format!("{}/data-saver", chapter_top_dir);
    std::fs::create_dir_all(format!("{}/data-saver", chapter_top_dir))?;
    info!("chapter dir created");
    let at_home = client
        .at_home()
        .server()
        .chapter_id(&chapter_id)
        .build()?
        .send()
        .await?;
    let http_client = get_reqwest_client(&client).await;
    verify_chapter_and_manga(&chapter_id, client_, &chapter_top_dir).await?;
    let mut files_: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data_saver;
    for filename in page_filenames {
        let path_to_use = format!("{}/{}", chapter_dir, &filename);
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home.base_url.join(&format!(
            "/{quality_mode}/{chapter_hash}/{page_filename}",
            quality_mode = "data-saver",
            chapter_hash = at_home.chapter.hash,
            page_filename = filename
        ))?;
        download_chapter_file(&http_client, page_url, &path_to_use, &mut files_, &filename).await?;
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "downloaded" : files_
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
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