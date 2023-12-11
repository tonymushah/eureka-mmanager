mod with_manga;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use mangadex_api::utils::download::cover::CoverQuality;
use mangadex_api::{v5::MangaDexClient, HttpClientRef};
use mangadex_api_schema_rust::v5::CoverAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use uuid::Uuid;

use crate::core::{Error, ManagerCoreResult};
use crate::server::traits::{AccessDownloadTasks, AccessHistory};
use crate::settings::files_dirs::DirsOptions;
use crate::settings::{self};
use crate::utils::cover::CoverUtilsWithId;

use mangadex_api_types_rust::error::Result as MangadexResult;

pub use with_manga::{AccessCoverDownloadWithManga, CoverDownloadWithManga};

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
        let http_client = self.http_client.read().await.client.clone();
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
                let files = File::create(json_cover)?;
                serde_json::from_str::<ApiData<ApiObject<CoverAttributes>>>(bytes_string.as_str())?;
                {
                    let mut writer = BufWriter::new(files);
                    writer.write_all(&bytes)?;
                    writer.flush()?;
                }
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
        let res: ManagerCoreResult<(String, MangadexResult<bytes::Bytes>)> = task_manager
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

        if let Ok(bytes) = bytes_ {
            {
                let file = File::create(file_path)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(&bytes)?;
                writer.flush()?;
            }

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
        let res: ManagerCoreResult<(String, MangadexResult<bytes::Bytes>)> = task_manager
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

        if let Ok(bytes) = bytes_ {
            {
                let file = File::create(file_path)?;
                let mut writer = BufWriter::new(file);
                writer.write_all(&bytes)?;
                writer.flush()?;
            }
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

impl From<CoverUtilsWithId> for CoverDownload {
    fn from(value: CoverUtilsWithId) -> Self {
        Self {
            dirs_options: value.cover_utils.dirs_options,
            http_client: value.cover_utils.http_client_ref,
            cover_id: value.cover_id,
        }
    }
}

impl<'a> From<&'a CoverUtilsWithId> for CoverDownload {
    fn from(value: &'a CoverUtilsWithId) -> Self {
        Self {
            dirs_options: value.cover_utils.dirs_options.clone(),
            http_client: value.cover_utils.http_client_ref.clone(),
            cover_id: value.cover_id,
        }
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
