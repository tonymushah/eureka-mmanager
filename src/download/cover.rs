// Imports used for downloading the cover to a file.
// They are not used because we're just printing the raw bytes.
use std::fs::File;
use std::io::Write;
use std::sync::Arc;

use mangadex_api::utils::download::cover::CoverQuality;
use mangadex_api::{v5::MangaDexClient, HttpClientRef};
use mangadex_api_schema_rust::v5::CoverAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

use crate::core::{Error, ManagerCoreResult};
use crate::server::traits::{AccessDownloadTasks, AccessHistory};
use crate::settings::files_dirs::DirsOptions;
use crate::settings::{self};
use crate::utils::cover::CoverUtilsWithId;
use crate::utils::manga::MangaUtilsWithMangaId;

use super::manga::MangaDownload;

#[derive(Clone)]
pub struct CoverDownload {
    pub dirs_options: Arc<DirsOptions>,
    pub http_client: HttpClientRef,
    pub cover_id: Uuid,
}

impl CoverDownload {
    pub fn new(cover_id: Uuid, dirs_options: Arc<DirsOptions>, http_client: HttpClientRef) -> Self {
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
        task_manager
            .lock_spawn_with_data(async move {
                
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
                let mut files = File::create(json_cover)?;
                serde_json::from_str::<ApiData<ApiObject<CoverAttributes>>>(bytes_string.as_str())?;
                files.write_all(&bytes)?;
                Ok(())
            })
            .await?
    }
    pub async fn download<'a, D>(
        &self,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let cover_id = self.cover_id;
        let res: ManagerCoreResult<(String, Option<bytes::Bytes>)> = task_manager
            .lock_spawn_with_data(async move {
                let resp = client
                    .download()
                    .cover()
                    .build()?
                    .via_cover_id(cover_id)
                    .await?;
                Ok(resp)
            })
            .await?;
        let (filename, bytes_) = res?;

        // This is where you would download the file but for this example, we're just printing the raw data.
        let files_dirs = settings::files_dirs::DirsOptions::new()?;
        let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());

        if let Some(bytes) = bytes_ {
            let mut file = File::create(file_path)?;
            let _ = file.write_all(&bytes);

            self.download_cover_data(task_manager).await?;

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
    pub async fn download_with_quality<'a, D>(
        &self,
        quality: CoverQuality,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let cover_id = self.cover_id;
        let res: ManagerCoreResult<(String, Option<bytes::Bytes>)> = task_manager
            .lock_spawn_with_data(async move {
                let resp = client
                    .download()
                    .cover()
                    .quality(quality)
                    .build()?
                    .via_cover_id(cover_id)
                    .await?;
                Ok(resp)
            })
            .await?;
        let (filename, bytes_) = res?;

        // This is where you would download the file but for this example, we're just printing the raw data.
        let files_dirs = settings::files_dirs::DirsOptions::new()?;
        let file_path = files_dirs.covers_add(format!("images/{}", filename.as_str()).as_str());

        if let Some(bytes) = bytes_ {
            let mut file = File::create(file_path)?;
            let _ = file.write_all(&bytes);
            self.download_cover_data(task_manager).await?;
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
}

impl TryFrom<CoverUtilsWithId> for CoverDownload {
    type Error = uuid::Error;
    fn try_from(value: CoverUtilsWithId) -> Result<Self, Self::Error> {
        Ok(Self {
            dirs_options: value.cover_utils.dirs_options,
            http_client: value.cover_utils.http_client_ref,
            cover_id: Uuid::try_from(value.cover_id.as_str())?,
        })
    }
}

impl<'a> TryFrom<&'a CoverUtilsWithId> for CoverDownload {
    type Error = uuid::Error;
    fn try_from(value: &'a CoverUtilsWithId) -> Result<Self, Self::Error> {
        Ok(Self {
            dirs_options: value.cover_utils.dirs_options.clone(),
            http_client: value.cover_utils.http_client_ref.clone(),
            cover_id: Uuid::try_from(value.cover_id.as_str())?,
        })
    }
}

#[async_trait::async_trait]
pub trait AccessCoverDownload: AccessDownloadTasks + AccessHistory + Sized + Send + Sync {
    async fn download_json_data<'a>(
        &'a mut self,
        cover_download: &'a CoverDownload,
    ) -> ManagerCoreResult<()> {
        cover_download.download_cover_data(self).await
    }
    async fn download<'a>(
        &'a mut self,
        cover_download: &'a CoverDownload,
    ) -> ManagerCoreResult<serde_json::Value> {
        cover_download.download(self).await
    }
    async fn download_with_quality<'a>(
        &'a mut self,
        cover_download: &'a CoverDownload,
        quality: CoverQuality,
    ) -> ManagerCoreResult<serde_json::Value> {
        cover_download.download_with_quality(quality, self).await
    }
}

#[derive(Clone)]
pub struct CoverDownloadWithManga {
    pub dirs_options: Arc<DirsOptions>,
    pub http_client: HttpClientRef,
    pub manga_id: Uuid,
}

impl CoverDownloadWithManga {
    pub fn new(manga_id: Uuid, dirs_options: Arc<DirsOptions>, http_client: HttpClientRef) -> Self {
        Self {
            dirs_options,
            http_client,
            manga_id,
        }
    }
    pub async fn download<'a, D>(
        &self,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let manga = client
            .manga()
            .get()
            .manga_id(self.manga_id)
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
                    format!("no cover art found for manga {}", self.manga_id),
                )))
            }
        }
        .id;
        CoverDownload::new(
            cover_id,
            self.dirs_options.clone(),
            client.get_http_client(),
        )
        .download(task_manager)
        .await
    }
    pub async fn download_with_quality<'a, D>(
        &self,
        quality: CoverQuality,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let manga = client
            .manga()
            .get()
            .manga_id(self.manga_id)
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
                    format!("no cover art found for manga {}", self.manga_id),
                )))
            }
        }
        .id;
        CoverDownload::new(
            cover_id,
            self.dirs_options.clone(),
            client.get_http_client(),
        )
        .download_with_quality(quality, task_manager)
        .await
    }
    pub async fn all_cover_download<'a, D>(
        &self,
        limit: u32,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());

        let covers = client
            .cover()
            .list()
            .add_manga_id(self.manga_id)
            .limit(limit)
            .build()?
            .send()
            .await?;
        let mut vecs: Vec<String> = Vec::new();
        for cover_to_use in covers.data {
            if (CoverDownload::new(
                cover_to_use.id,
                self.dirs_options.clone(),
                client.get_http_client(),
            )
            .download(task_manager)
            .await)
                .is_ok()
            {
                vecs.push(format!("{}", cover_to_use.id.hyphenated()));
            }
        }
        let jsons = serde_json::json!({
            "result" : "ok",
            "id": self.manga_id,
            "type" : "collection",
            "downloaded" : vecs
        });
        let mut files = File::create(format!("covers/lists/{}.json", self.manga_id))?;
        files.write_all(jsons.to_string().as_bytes())?;

        Ok(jsons)
    }
}

impl TryFrom<MangaUtilsWithMangaId> for CoverDownloadWithManga {
    type Error = uuid::Error;
    fn try_from(value: MangaUtilsWithMangaId) -> Result<Self, Self::Error> {
        Ok(Self {
            dirs_options: value.manga_utils.dirs_options,
            http_client: value.manga_utils.http_client_ref,
            manga_id: Uuid::try_from(value.manga_id.as_str())?,
        })
    }
}

impl<'a> TryFrom<&'a MangaUtilsWithMangaId> for CoverDownloadWithManga {
    type Error = uuid::Error;
    fn try_from(value: &'a MangaUtilsWithMangaId) -> Result<Self, Self::Error> {
        Ok(Self {
            dirs_options: value.manga_utils.dirs_options.clone(),
            http_client: value.manga_utils.http_client_ref.clone(),
            manga_id: Uuid::try_from(value.manga_id.as_str())?,
        })
    }
}

impl From<MangaDownload> for CoverDownloadWithManga {
    fn from(value: MangaDownload) -> Self {
        Self {
            dirs_options: value.dirs_options,
            http_client: value.http_client,
            manga_id: value.manga_id,
        }
    }
}

impl<'a> From<&'a MangaDownload> for CoverDownloadWithManga {
    fn from(value: &'a MangaDownload) -> Self {
        Self {
            dirs_options: value.dirs_options.clone(),
            http_client: value.http_client.clone(),
            manga_id: value.manga_id,
        }
    }
}

#[async_trait::async_trait]
pub trait AccessCoverDownloadWithManga:
    AccessDownloadTasks + AccessHistory + Sized + Send + Sync
{
    async fn download<'a>(
        &'a mut self,
        cover_download: &'a CoverDownloadWithManga,
    ) -> ManagerCoreResult<serde_json::Value> {
        cover_download.download(self).await
    }
    async fn download_with_quality<'a>(
        &'a mut self,
        cover_download: &'a CoverDownloadWithManga,
        quality: CoverQuality,
    ) -> ManagerCoreResult<serde_json::Value> {
        cover_download.download_with_quality(quality, self).await
    }
    async fn all_cover_download<'a>(
        &'a mut self,
        cover_download: &'a CoverDownloadWithManga,
        limit: u32,
    ) -> ManagerCoreResult<serde_json::Value> {
        cover_download.all_cover_download(limit, self).await
    }
}
