// Imports used for downloading the pages to a file.
// They are not used because we're just printing the raw bytes.
use log::{info, warn};
use mangadex_api::v5::MangaDexClient;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

use crate::{
    settings::{
        self, commit_rel, file_history::HistoryEntry, insert_in_history, remove_in_history,
    },
    utils::{self, chapter::{is_chapter_manga_there, patch_manga_by_chapter}, send_request},
};

pub async fn download_chapter(chapter_id: &str) -> anyhow::Result<serde_json::Value> {
    let chapter_id = Uuid::parse_str(chapter_id)?;
    let history_entry =
        HistoryEntry::new(chapter_id, mangadex_api_types::RelationshipType::Chapter);
    match insert_in_history(&history_entry) {
        Ok(_) => (),
        Err(error) => {
            if error.kind() != std::io::ErrorKind::AlreadyExists {
                return Err(anyhow::Error::new(error));
            }
        }
    };
    commit_rel(history_entry.get_data_type())?;
    let client = MangaDexClient::default();
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let chapter_top_dir = files_dirs.chapters_add(chapter_id.hyphenated().to_string().as_str());
    let chapter_dir = format!("{}/data", chapter_top_dir);
    std::fs::create_dir_all(format!("{}", chapter_dir))?;
    info!("chapter dir created");
    let at_home = client
        .at_home()
        .server()
        .chapter_id(&chapter_id)
        .build()?
        .send()
        .await?;
    let http_client = reqwest::Client::new();
    // puting chapter data in a json data
    if Path::new(format!("{}/data.json", chapter_top_dir).as_str()).exists() == false {
        let get_chapter = send_request(http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated().to_string())), 5).await?;
        let bytes_ = get_chapter.bytes().await?;
        let mut chapter_data = File::create(format!("{}/data.json", chapter_top_dir))?;
        chapter_data.write_all(&bytes_)?;
        info!("created data.json");
    }
    match is_chapter_manga_there(format!("{}", chapter_id)) {
        Ok(data) => {
            if data == false {
                patch_manga_by_chapter(format!("{}", chapter_id)).await?;
            }
        }
        Err(e) => {
            let error = e.to_string();
            warn!("Warning {}!", error);
            patch_manga_by_chapter(format!("{}", chapter_id)).await?;
        }
    }
    let mut files_: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data;
    for filename in page_filenames {
        let path_to_use = format!("{}/{}", chapter_dir, &filename);
        let path_to_use_clone = path_to_use.clone();
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home.base_url.join(&format!(
            "/{quality_mode}/{chapter_hash}/{page_filename}",
            quality_mode = "data",
            chapter_hash = at_home.chapter.hash,
            page_filename = filename
        ))?;
        match utils::send_request(http_client.get(page_url), 5).await {
            Ok(res) => {
                match File::open(path_to_use) {
                    Ok(file) => {
                        let res_length = res.content_length();
                        // The data should be streamed rather than downloading the data all at once.
                        if Path::new(path_to_use_clone.as_str()).exists() == false
                            || res_length.is_none() == true
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
                                    let mut file = File::create(path_to_use_clone)?;
                                    match file.write_all(&bytes) {
                                        core::result::Result::Err(error) => {
                                            //info!(" at file write_all : {}", error.to_string());
                                            return Err(anyhow::Error::new(error));
                                        }
                                        core::result::Result::Ok(_) => {
                                            info!("downloaded {} ", &filename);
                                            files_.push(format!("{}", &filename));
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
                                let mut file = File::create(path_to_use_clone)?;
                                match file.write_all(&bytes) {
                                    core::result::Result::Err(error) => {
                                        //info!(" at file write_all : {}", error.to_string());
                                        return Err(anyhow::Error::new(error));
                                    }
                                    core::result::Result::Ok(_) => {
                                        info!("downloaded {} ", &filename);
                                        files_.push(format!("{}", &filename));
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
pub async fn download_chapter_saver(chapter_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
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

    let http_client = reqwest::Client::new();
    // puting chapter data in a json data
    if Path::new(format!("{}/data.json", chapter_top_dir).as_str()).exists() == false {
        let get_chapter = send_request(http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated().to_string())), 5).await?;
        let bytes_ = get_chapter.bytes().await?;
        let mut chapter_data = File::create(format!("{}/data.json", chapter_top_dir))?;
        chapter_data.write_all(&bytes_)?;
        info!("created data.json");
    }
    match is_chapter_manga_there(format!("{}", chapter_id)) {
        Ok(data) => {
            if data == false {
                patch_manga_by_chapter(format!("{}", chapter_id)).await?;
            }
        }
        Err(e) => {
            let error = e.to_string();
            warn!("Warning {}!", error);
            patch_manga_by_chapter(format!("{}", chapter_id)).await?;
        }
    }
    let mut files_: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data_saver;
    for filename in page_filenames {
        let path_to_use = format!("{}/{}", chapter_dir, &filename);
        let path_to_use_clone = format!("{}/{}", chapter_dir, &filename);
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home.base_url.join(&format!(
            "/{quality_mode}/{chapter_hash}/{page_filename}",
            quality_mode = "data-saver",
            chapter_hash = at_home.chapter.hash,
            page_filename = filename
        ))?;
        match utils::send_request(http_client.get(page_url), 5).await {
            Ok(res) => {
                match File::open(path_to_use) {
                    Ok(file) => {
                        let res_length = res.content_length();
                        // The data should be streamed rather than downloading the data all at once.
                        if Path::new(path_to_use_clone.as_str()).exists() == false
                            || res_length.is_none() == true
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
                                    let mut file = File::create(path_to_use_clone)?;
                                    match file.write_all(&bytes) {
                                        core::result::Result::Err(error) => {
                                            //info!(" at file write_all : {}", error.to_string());
                                            return Err(anyhow::Error::new(error));
                                        }
                                        core::result::Result::Ok(_) => {
                                            info!("downloaded {} ", &filename);
                                            files_.push(format!("{}", &filename));
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
                                let mut file = File::create(path_to_use_clone)?;
                                match file.write_all(&bytes) {
                                    core::result::Result::Err(error) => {
                                        //info!(" at file write_all : {}", error.to_string());
                                        return Err(anyhow::Error::new(error));
                                    }
                                    core::result::Result::Ok(_) => {
                                        info!("downloaded {} ", &filename);
                                        files_.push(format!("{}", &filename));
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
