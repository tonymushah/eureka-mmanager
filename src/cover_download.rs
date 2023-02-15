// Imports used for downloading the cover to a file.
// They are not used because we're just printing the raw bytes.
use std::fs::File;
use std::io::Write;

use anyhow::Ok;
use reqwest::Url;
use uuid::Uuid;
use mangadex_api::types::RelationshipType;
use mangadex_api::v5::MangaDexClient;
use mangadex_api::CDN_URL;
use log::info;

use crate::{settings::{self}, utils};

pub async fn cover_download_by_manga_id(manga_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let manga_id = Uuid::parse_str(manga_id)?;
    let manga = client
        .manga()
        .get()
        .manga_id(&manga_id)
        .build()?
        .send()
        .await?;
    let cover_id = match manga
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::CoverArt){
            Some(data) => data,
            None => {
                return Err(anyhow::Error::msg("no cover art found for manga"))
            }
        }
        .id;
    cover_download_by_cover(cover_id.to_string().as_str()).await
}

pub async fn cover_download_quality_by_manga_id(manga_id: &str, quality:  u32) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let manga_id = Uuid::parse_str(manga_id)?;
    let manga = client
        .manga()
        .get()
        .manga_id(&manga_id)
        .build()?
        .send()
        .await?;

    let cover_id = match manga
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::CoverArt) {
            Some(data) => data,
            None => {
                return Err(anyhow::Error::msg("no cover art found for manga"))
            }
        }
        .id;

    cover_download_quality_by_cover(cover_id.to_string().as_str(), quality).await
}

pub async fn cover_download_by_cover(cover_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let cover = client
        .cover()
        .get()
        .cover_id(&Uuid::parse_str(cover_id)?)
        .build()?
        .send()
        .await?;

    let http_client = reqwest::Client::new();
    
    let manga_id = match cover
        .data
        .relationships
        .iter()
        .find(
            | related 
            | related.type_ == RelationshipType::Manga
        ){
            Some(data) => data,
            None => {
                return Err(anyhow::Error::msg("no manga found for cover"))
            }
        }
        .id;
    // This uses the best quality image.
    // To use smaller, thumbnail-sized images, append any of the following:
    //
    // - .512.jpg
    // - .256.jpg
    //
    // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
    let cover_url = Url::parse(&format!(
            "{}/covers/{}/{}",
            CDN_URL, manga_id, cover.data.attributes.file_name
        ))?;

    

    let res = utils::send_request(http_client.get(cover_url), 5).await?;
    // The data should be streamed rather than downloading the data all at once.
    let bytes = res.bytes().await?;
    let filename = cover.data.attributes.file_name;
    // This is where you would download the file but for this example, we're just printing the raw data.
        let files_dirs = settings::files_dirs::DirsOptions::new()?;
        let file_dirs_clone = files_dirs.clone();
        let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());
        let json_cover = file_dirs_clone.covers_add(format!("{}.json", cover_id).as_str());

        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        let mut files = File::create(json_cover)?;

        let resps = utils::send_request(http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_id)), 5).await?;

        files.write_all(&resps.bytes().await?)?;        
    Ok(serde_json::json!({
        "result" : "ok",
        "type": "cover",
        "downloded" : cover_id
    }))
}

pub async fn cover_download_quality_by_cover(cover_id: &str, quality:  u32) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let cover = client
        .cover()
        .get()
        .cover_id(&Uuid::parse_str(cover_id)?)
        .build()?
        .send()
        .await?;
    let http_client = reqwest::Client::new();
    let manga_id = match cover
        .data
        .relationships
        .iter()
        .find(
            | related 
            | related.type_ == RelationshipType::Manga
        ) {
            Some(data) => data,
            None => {
                return Err(anyhow::Error::msg("can't find manga for cover_art"))
            }
        }
        .id;
    if quality == 256 || quality == 512 {
        // This uses the best quality image.
        // To use smaller, thumbnail-sized images, append any of the following:
        //
        // - .512.jpg
        // - .256.jpg
        //
        // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
        let cover_url = Url::parse(&format!(
                "{}/covers/{}/{}",
                CDN_URL, manga_id, format!("{}.{}.jpg", cover.data.attributes.file_name, quality)
            ))?;

        let res = utils::send_request(http_client.get(cover_url), 5).await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;
        let filename = cover.data.attributes.file_name;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let files_dirs = settings::files_dirs::DirsOptions::new()?;
        let file_dirs_clone = files_dirs.clone();
        let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());
        let json_cover = file_dirs_clone.covers_add(format!("{}.json", cover_id).as_str());

        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        let mut files = File::create(json_cover)?;

        let resps = utils::send_request(http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_id)), 5).await?;

        files.write_all(&resps.bytes().await?)?;

        info!("downloaded {}", filename.as_str());
        //    info!("downloaded {}", filename.as_str());
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    }else{
        Err(anyhow::Error::msg("not a valid size"))
    }
}

pub async fn all_covers_download_quality_by_manga_id(manga_id: &str, limit: u32) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let manga_id = Uuid::parse_str(manga_id)?;

    let covers = client
        .cover()
        .list()
        .add_manga_id(&manga_id)
        .limit(limit)
        .build()?
        .send()
        .await?;
    let http_client = reqwest::Client::new();
    let mut vecs : Vec<String> = Vec::new();
    for cover_to_use in covers.data{
        let cover_url = Url::parse(&format!(
                "{}/covers/{}/{}",
                CDN_URL, manga_id, cover_to_use.attributes.file_name
            ))?;
        let res = utils::send_request(http_client.get(cover_url), 5).await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;
        let filename = cover_to_use.attributes.file_name;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let files_dirs = settings::files_dirs::DirsOptions::new()?;
        let file_dirs_clone = files_dirs.clone();
        let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());
        let json_cover = file_dirs_clone.covers_add(format!("{}.json", cover_to_use.id.hyphenated()).as_str());

        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        let mut files = File::create(json_cover)?;

        let resps = utils::send_request(http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_to_use.id.hyphenated())), 5).await?;

        files.write_all(&resps.bytes().await?)?;        

        vecs.push(format!("{}", cover_to_use.id.hyphenated()));
        info!("downloaded {}", filename.as_str());
    }
    let jsons = serde_json::json!({
        "result" : "ok",
        "id": manga_id,
        "type" : "collection",
        "downloaded" : vecs
    });
    let mut files = File::create(format!("covers/lists/{}.json", manga_id))?;
    files.write_all(jsons.to_string().as_bytes())?;
    
    Ok(jsons)
}
