// Imports used for downloading the cover to a file.
// They are not used because we're just printing the raw bytes.
use std::fs::File;
use std::io::Write;

use anyhow::Ok;
use log::info;
use mangadex_api::utils::download::cover::CoverQuality;
use mangadex_api::utils::get_reqwest_client;
use mangadex_api::{v5::MangaDexClient, HttpClientRef};
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

use crate::{
    settings::{self},
    utils,
};

pub async fn cover_download_by_cover(
    cover_id: &str,
    client: HttpClientRef,
) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);
    let (filename, bytes_) = client
        .download()
        .cover()
        .build()?
        .via_cover_id(Uuid::parse_str(cover_id)?)
        .await?;
    // This is where you would download the file but for this example, we're just printing the raw data.
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let file_dirs_clone = files_dirs.clone();
    let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());
    let json_cover = file_dirs_clone.covers_add(format!("{}.json", cover_id).as_str());

    if let Some(bytes) = bytes_ {
        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        let mut files = File::create(json_cover)?;
        let http_client = get_reqwest_client(&client).await;
        let resps = utils::send_request(
            http_client.get(format!(
                "{}/cover/{}",
                mangadex_api::constants::API_URL,
                cover_id
            )),
            5,
        )
        .await?;

        files.write_all(&resps.bytes().await?)?;
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    } else {
        Err(anyhow::Error::msg(format!(
            "Empty byte found for {filename}"
        )))
    }
}

pub async fn cover_download_by_manga_id(
    manga_id: &str,
    client: HttpClientRef,
) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);
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
        .find(|related| related.type_ == RelationshipType::CoverArt)
    {
        Some(data) => data,
        None => return Err(anyhow::Error::msg("no cover art found for manga")),
    }
    .id;
    cover_download_by_cover(cover_id.to_string().as_str(), client.get_http_client()).await
}

pub async fn cover_download_quality_by_manga_id(
    manga_id: &str,
    quality: CoverQuality,
    client: HttpClientRef,
) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);
    let cover_id = Uuid::parse_str(manga_id)?;
    // The data should be streamed rather than downloading the data all at once.
    let (filename, bytes_) = client
        .download()
        .cover()
        .quality(quality)
        .build()?
        .via_manga_id(cover_id)
        .await?;
    // This is where you would download the file but for this example, we're just printing the raw data.
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let file_dirs_clone = files_dirs.clone();
    let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());
    let json_cover = file_dirs_clone.covers_add(format!("{}.json", cover_id).as_str());

    if let Some(bytes) = bytes_ {
        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        let mut files = File::create(json_cover)?;
        let http_client = get_reqwest_client(&client).await;
        let resps = utils::send_request(
            http_client.get(format!(
                "{}/cover/{}",
                mangadex_api::constants::API_URL,
                cover_id
            )),
            5,
        )
        .await?;

        files.write_all(&resps.bytes().await?)?;
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    } else {
        Err(anyhow::Error::msg(format!(
            "Empty byte found for {filename}"
        )))
    }
}

pub async fn cover_download_quality_by_cover(
    cover_id: &str,
    quality: CoverQuality,
    client: HttpClientRef,
) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);

    let (filename, bytes_) = client
        .download()
        .cover()
        .quality(quality)
        .build()?
        .via_cover_id(Uuid::parse_str(cover_id)?)
        .await?;
    // This is where you would download the file but for this example, we're just printing the raw data.
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let file_dirs_clone = files_dirs.clone();
    let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());
    let json_cover = file_dirs_clone.covers_add(format!("{}.json", cover_id).as_str());

    if let Some(bytes) = bytes_ {
        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        let mut files = File::create(json_cover)?;
        let http_client = get_reqwest_client(&client).await;
        let resps = utils::send_request(
            http_client.get(format!(
                "{}/cover/{}",
                mangadex_api::constants::API_URL,
                cover_id
            )),
            5,
        )
        .await?;

        files.write_all(&resps.bytes().await?)?;
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    } else {
        Err(anyhow::Error::msg(format!(
            "Empty byte found for {filename}"
        )))
    }
}

pub async fn all_covers_download_quality_by_manga_id(
    manga_id: &str,
    limit: u32,
    client: HttpClientRef,
) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);
    let manga_id = Uuid::parse_str(manga_id)?;

    let covers = client
        .cover()
        .list()
        .add_manga_id(&manga_id)
        .limit(limit)
        .build()?
        .send()
        .await?;
    let http_client = get_reqwest_client(&client).await;
    let mut vecs: Vec<String> = Vec::new();
    for cover_to_use in covers.data {
        // The data should be streamed rather than downloading the data all at once.
        let (filename, bytes_) = client
            .download()
            .cover()
            .build()?
            .via_cover_api_object(cover_to_use.clone())
            .await?;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let files_dirs = settings::files_dirs::DirsOptions::new()?;
        let file_dirs_clone = files_dirs.clone();
        let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());
        let json_cover =
            file_dirs_clone.covers_add(format!("{}.json", cover_to_use.id.hyphenated()).as_str());

        if let Some(bytes) = bytes_ {
            let mut file = File::create(file_path)?;
            let _ = file.write_all(&bytes);
            let mut files = File::create(json_cover)?;

            let resps = utils::send_request(
                http_client.get(format!(
                    "{}/cover/{}",
                    mangadex_api::constants::API_URL,
                    cover_to_use.id.hyphenated()
                )),
                5,
            )
            .await?;

            files.write_all(&resps.bytes().await?)?;

            vecs.push(format!("{}", cover_to_use.id.hyphenated()));
            info!("downloaded {}", filename.as_str());
        }
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
