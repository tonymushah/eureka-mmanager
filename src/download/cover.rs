mod with_manga;

use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use mangadex_api::utils::download::cover::CoverQuality;
use mangadex_api::{v5::MangaDexClient, HttpClientRef};
use mangadex_api_schema_rust::v5::CoverData;
use mangadex_api_types_rust::ReferenceExpansionResource;
use uuid::Uuid;

use crate::core::ManagerCoreResult;
use crate::server::traits::{AccessDownloadTasks, AccessHistory};
use crate::settings::files_dirs::DirsOptions;
use crate::utils::cover::CoverUtilsWithId;
use crate::utils::ExtractData;

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
    pub async fn download_cover_data<'a, D>(
        &self,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<CoverData>
    where
        D: AccessDownloadTasks,
    {
        let cover_utils: CoverUtilsWithId = From::from(self);
        let id = self.cover_id;
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        task_manager
            .lock_spawn_with_data(async move {
                let cover = client
                    .cover()
                    .cover_id(id)
                    .get()
                    .include(ReferenceExpansionResource::Manga)
                    .include(ReferenceExpansionResource::User)
                    .send()
                    .await?;
                let mut writer = cover_utils.get_buf_writer()?;
                serde_json::to_writer(&mut writer, &cover)?;
                writer.flush()?;
                Ok(cover)
            })
            .await?
    }
    pub async fn download<'a, D>(&self, task_manager: &'a mut D) -> ManagerCoreResult<CoverData>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let cover_id = self.cover_id;
        let (filename, bytes_) = task_manager
            .lock_spawn_with_data(async move {
                let resp = client
                    .download()
                    .cover()
                    .build()?
                    .via_cover_id(cover_id)
                    .await?;
                Ok::<(String, MangadexResult<bytes::Bytes>), crate::Error>(resp)
            })
            .await??;
        let cover_utils: CoverUtilsWithId = From::from(self);

        let bytes = bytes_?;

        let mut writer = BufWriter::new(File::create(cover_utils.get_image_path_(filename)?)?);
        writer.write_all(&bytes)?;
        writer.flush()?;

        self.download_cover_data(task_manager).await
    }
    pub async fn download_with_quality<'a, D>(
        &self,
        quality: CoverQuality,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<CoverData>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let cover_id = self.cover_id;
        let (filename, bytes_) = task_manager
            .lock_spawn_with_data(async move {
                let resp = client
                    .download()
                    .cover()
                    .quality(quality)
                    .build()?
                    .via_cover_id(cover_id)
                    .await?;
                Ok::<(String, MangadexResult<bytes::Bytes>), crate::Error>(resp)
            })
            .await??;

        let cover_utils: CoverUtilsWithId = From::from(self);

        let bytes = bytes_?;

        let mut writer = BufWriter::new(File::create(cover_utils.get_image_path_(filename)?)?);
        writer.write_all(&bytes)?;
        writer.flush()?;
        self.download_cover_data(task_manager).await
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
    ) -> ManagerCoreResult<CoverData> {
        cover_download.download_cover_data(self).await
    }
    async fn download<'a>(
        &'a mut self,
        cover_download: &'a CoverDownload,
    ) -> ManagerCoreResult<CoverData> {
        cover_download.download(self).await
    }
    async fn download_with_quality<'a>(
        &'a mut self,
        cover_download: &'a CoverDownload,
        quality: CoverQuality,
    ) -> ManagerCoreResult<CoverData> {
        cover_download.download_with_quality(quality, self).await
    }
}
