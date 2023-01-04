// Imports used for downloading the pages to a file.
// They are not used because we're just printing the raw bytes.
use log::{info, warn};
use mangadex_api::v5::MangaDexClient;
use serde_json::json;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use uuid::Uuid;

use crate::{settings, utils::{self, send_request, is_chapter_manga_there, patch_manga_by_chapter}};

pub async fn download_chapter(chapter_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let chapter_id = Uuid::parse_str(chapter_id)?;
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
        },
        Err(e) => {
            let error = e.to_string();
            warn!("Warning {}!", error);
        }
    }
    let mut files_: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data;
    for filename in page_filenames {
        let path_to_use = format!("{}/{}", chapter_dir, &filename);
        let path_to_use_clone = path_to_use.clone();
        let mut file = File::create(path_to_use)?;
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home.base_url.join(&format!(
            "/{quality_mode}/{chapter_hash}/{page_filename}",
            quality_mode = "data",
            chapter_hash = at_home.chapter.hash,
            page_filename = filename
        ))?;
        match utils::send_request(http_client.get(page_url), 5).await {
            Ok(res) => {
                // The data should be streamed rather than downloading the data all at once.
                if Path::new(path_to_use_clone.as_str()).exists() == false
                    || file.metadata()?.len()
                        != (match res.content_length() {
                            Some(f) => f,
                            None => {
                                continue;
                            }
                        })
                {
                    let bytes = res.bytes().await?;
                    // This is where you would download the file but for this example,
                    // we're just printing the raw data.

                    let _ = file.write_all(&bytes)?;
                }
                info!("downloaded {} ", &filename);

                files_.push(format!("{}", &filename));
            }
            Err(_) => {
                failed.push(format!("{}", &filename));
            }
        };
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "failed" : failed,
        "downloaded" : files_
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
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
        let get_chapter = http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated().to_string())).send().await?;
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
        },
        Err(e) => {
            let error = e.to_string();
            warn!("Warning {}!", error);
        }
    }
    let mut files_: Vec<String> = Vec::new();
    let mut failed: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data_saver;
    for filename in page_filenames {
        let path_to_use = format!("{}/{}", chapter_dir, &filename);
        let path_to_use_clone = format!("{}/{}", chapter_dir, &filename);
        let mut file = File::create(path_to_use)?;
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home.base_url.join(&format!(
            "/{quality_mode}/{chapter_hash}/{page_filename}",
            quality_mode = "data-saver",
            chapter_hash = at_home.chapter.hash,
            page_filename = filename
        ))?;

        match utils::send_request(http_client.get(page_url), 5).await {
            Ok(res) => {
                // The data should be streamed rather than downloading the data all at once.
                if Path::new(path_to_use_clone.as_str()).exists() == false
                    || file.metadata()?.len()
                        != (match res.content_length() {
                            Some(f) => f,
                            None => {
                                continue;
                            }
                        })
                {
                    let bytes = res.bytes().await?;
                    // This is where you would download the file but for this example,
                    // we're now sending the data to the right file.

                    let _ = file.write_all(&bytes)?;
                }
                info!("downloaded {} ", &filename);

                files_.push(format!("{}", &filename));
            }
            Err(_) => {
                failed.push(format!("{}", &filename));
            }
        };
        files_.push(format!("{}", &filename));
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "failed" : failed,
        "downloaded" : files_
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
    Ok(jsons)
}

pub mod default {
    use log::info;
    use mangadex_api::v5::MangaDexClient;
    use serde_json::json;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use uuid::Uuid;

    use crate::{settings, utils};
    pub async fn download_chapter(chapter_id: &str) -> anyhow::Result<serde_json::Value> {
        let client = MangaDexClient::default();
        let files_dirs = settings::files_dirs::DirsOptions::new()?;
        let chapter_id = Uuid::parse_str(chapter_id)?;
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
            let get_chapter = http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated().to_string())).send().await?;
            let bytes_ = get_chapter.bytes().await?;
            let mut chapter_data = File::create(format!("{}/data.json", chapter_top_dir))?;
            chapter_data.write_all(&bytes_)?;
            info!("created data.json");
        }
        let mut files_: Vec<String> = Vec::new();
        // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
        let page_filenames = at_home.chapter.data;
        for filename in page_filenames {
            // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
            let page_url = at_home.base_url.join(&format!(
                "/{quality_mode}/{chapter_hash}/{page_filename}",
                quality_mode = "data",
                chapter_hash = at_home.chapter.hash,
                page_filename = filename
            ))?;
            let res = utils::send_request(http_client.get(page_url), 5).await?;
            // The data should be streamed rather than downloading the data all at once.
            let bytes = res.bytes().await?;

            // This is where you would download the file but for this example,
            // we're just printing the raw data.
            let mut file = File::create(format!("{}/{}", chapter_dir, &filename))?;
            let _ = file.write_all(&bytes);
            info!("downloaded {} ", &filename);
            files_.push(format!("{}", &filename));
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
            let get_chapter = http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated().to_string())).send().await?;
            let bytes_ = get_chapter.bytes().await?;
            let mut chapter_data = File::create(format!("{}/data.json", chapter_top_dir))?;
            chapter_data.write_all(&bytes_)?;
            info!("created data.json");
        }
        let mut files_: Vec<String> = Vec::new();
        // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
        let page_filenames = at_home.chapter.data_saver;
        for filename in page_filenames {
            // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
            let page_url = at_home.base_url.join(&format!(
                "/{quality_mode}/{chapter_hash}/{page_filename}",
                quality_mode = "data-saver",
                chapter_hash = at_home.chapter.hash,
                page_filename = filename
            ))?;

            let res = utils::send_request(http_client.get(page_url), 5).await?;
            // The data should be streamed rather than downloading the data all at once.
            let bytes = res.bytes().await?;

            // This is where you would download the file but for this example,
            // we're just printing the raw data.
            let mut file = File::create(format!("{}/{}", chapter_dir, &filename))?;
            let _ = file.write_all(&bytes);
            info!("downloaded {} ", &filename);
            files_.push(format!("{}", &filename));
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
}
