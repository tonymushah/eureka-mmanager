// Imports used for downloading the cover to a file.
// They are not used because we're just printing the raw bytes.
use std::fs::File;
use std::io::Write;

use log::info;
use mangadex_api::utils::download::cover::CoverQuality;
use mangadex_api::{v5::MangaDexClient, HttpClientRef};
use mangadex_api_schema_rust::v5::CoverAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

use crate::core::{Error, ManagerCoreResult};
use crate::server::traits::AccessDownloadTasks;
use crate::settings::files_dirs::DirsOptions;
use crate::settings::{self};

#[derive(Clone)]
pub struct CoverDownload {
    pub dirs_options: DirsOptions,
    pub http_client: HttpClientRef,
    pub cover_id: Uuid,
}

impl CoverDownload {
    pub fn new(cover_id: Uuid, dirs_options: DirsOptions, http_client: HttpClientRef) -> Self {
        Self {
            dirs_options,
            http_client,
            cover_id,
        }
    }
    pub async fn download_cover_data<'a, D>(&self, task_manager: &'a mut D) -> ManagerCoreResult<()>
    where
        D: AccessDownloadTasks,
    {
        let cover_id = self.cover_id.to_string();
        let json_cover = self
            .dirs_options
            .covers_add(format!("{}.json", cover_id).as_str());
        let http_client = self.http_client.lock().await.client.clone();
        task_manager.lock_spawn_with_data(async move {
            let mut files = File::create(json_cover)?;
            let resps = http_client
                .get(format!(
                    "{}/cover/{}",
                    mangadex_api::constants::API_URL,
                    cover_id
                ))
                .send()
                .await?;
            let bytes = resps.bytes().await?;
            let bytes_string = String::from_utf8(bytes.to_vec())?;
            serde_json::from_str::<ApiData<ApiObject<CoverAttributes>>>(bytes_string.as_str())?;
            files.write_all(&bytes)?;
            Ok(())
        }).await?
    }
}

pub async fn cover_download_by_cover(
    cover_id: &str,
    client: HttpClientRef,
) -> ManagerCoreResult<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);

    let (filename, bytes_) = client
        .download()
        .cover()
        .build()?
        .via_cover_id(Uuid::parse_str(cover_id)?)
        .await?;

    // This is where you would download the file but for this example, we're just printing the raw data.
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());

    if let Some(bytes) = bytes_ {
        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);

        download_cover_data(cover_id, client.get_http_client().clone()).await?;

        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    } else {
        Err(crate::core::Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Empty byte found for {filename}"),
        )))
    }
}

pub async fn cover_download_by_manga_id(
    manga_id: &str,
    client: HttpClientRef,
) -> ManagerCoreResult<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);
    let manga = client
        .manga()
        .get()
        .manga_id(Uuid::parse_str(manga_id)?)
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
        None => {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no cover art found for manga {manga_id}"),
            )))
        }
    }
    .id;
    cover_download_by_cover(cover_id.to_string().as_str(), client.get_http_client()).await
}

pub async fn cover_download_quality_by_manga_id(
    manga_id: &str,
    quality: CoverQuality,
    client: HttpClientRef,
) -> ManagerCoreResult<serde_json::Value> {
    let client = MangaDexClient::new_with_http_client_ref(client);
    let manga_id = Uuid::parse_str(manga_id)?;
    // The data should be streamed rather than downloading the data all at once.
    let cover = client
        .manga()
        .get()
        .manga_id(manga_id)
        .build()?
        .send()
        .await?;
    let cover_id = match cover
        .data
        .relationships
        .iter()
        .find(|rel| rel.type_ == RelationshipType::CoverArt)
    {
        Some(d) => d.id,
        None => {
            return Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "manga id not found",
            )));
        }
    };
    let (filename, bytes_) = client
        .download()
        .cover()
        .quality(quality)
        .build()?
        .via_cover_id(cover_id)
        .await?;
    // This is where you would download the file but for this example, we're just printing the raw data.
    let files_dirs = settings::files_dirs::DirsOptions::new()?;
    let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());

    if let Some(bytes) = bytes_ {
        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        download_cover_data(
            format!("{cover_id}").as_str(),
            client.get_http_client().clone(),
        )
        .await?;
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    } else {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Empty byte found for {filename}"),
        )))
    }
}

pub async fn cover_download_quality_by_cover(
    cover_id: &str,
    quality: CoverQuality,
    client: HttpClientRef,
) -> ManagerCoreResult<serde_json::Value> {
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
    let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());

    if let Some(bytes) = bytes_ {
        let mut file = File::create(file_path)?;
        let _ = file.write_all(&bytes);
        download_cover_data(cover_id, client.get_http_client().clone()).await?;
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    } else {
        Err(Error::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidData,
            format!("Empty byte found for {filename}"),
        )))
    }
}

pub async fn all_covers_download_quality_by_manga_id(
    manga_id: &str,
    limit: u32,
    client: HttpClientRef,
) -> ManagerCoreResult<serde_json::Value> {
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
        let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());

        if let Some(bytes) = bytes_ {
            let mut file = File::create(file_path)?;
            let _ = file.write_all(&bytes);
            download_cover_data(
                format!("{}", cover_to_use.id.clone()).as_str(),
                client.get_http_client().clone(),
            )
            .await?;

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
