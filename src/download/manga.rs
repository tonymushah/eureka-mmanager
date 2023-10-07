use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;

use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::v5::MangaAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use uuid::Uuid;

use crate::core::ManagerCoreResult;
use crate::server::traits::{AccessDownloadTasks, AccessHistory};
use crate::settings::files_dirs::DirsOptions;
use crate::utils::manga::MangaUtilsWithMangaId;
use crate::utils::send_request;

use super::cover::CoverDownloadWithManga;

#[derive(Clone)]
pub struct MangaDownload {
    pub dirs_options: Arc<DirsOptions>,
    pub http_client: HttpClientRef,
    pub manga_id: Uuid,
}

impl MangaDownload {
    pub fn new(manga_id: Uuid, dirs_options: Arc<DirsOptions>, http_client: HttpClientRef) -> Self {
        Self {
            dirs_options,
            http_client,
            manga_id,
        }
    }
    /// download the manga with the specified id
    pub async fn download_manga<'a, D>(&'a self, task_manager: &'a mut D) -> ManagerCoreResult<()>
    where
        D: AccessDownloadTasks,
    {
        let manga_utils: MangaUtilsWithMangaId = From::from(self);
        let id = format!("{}", self.manga_id);
        let http_client = self.http_client.lock().await.client.clone();
        let task : ManagerCoreResult<String> = task_manager.lock_spawn_with_data(async move {
            let resp = send_request(http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, id)), 5).await?;
            let bytes = resp.bytes().await?;
            let bytes_string = String::from_utf8(bytes.to_vec())?;
            serde_json::from_str::<ApiData<ApiObject<MangaAttributes>>>(bytes_string.as_str())?;
            {
                let file = (File::create(
                DirsOptions::new()?
                        .mangas_add(format!("{}.json", id).as_str())
                ))?;
                let mut writer = BufWriter::new(file);
                writer.write_all(&bytes)?;
                writer.flush()?;
            }
            Ok(id)
        }).await?;
        task?;
        let cover_download: CoverDownloadWithManga = From::from(self);
        if let Ok(is_there) = manga_utils.is_cover_there() {
            if !is_there {
                cover_download.download(task_manager).await?;
            }
        } else {
            cover_download.download(task_manager).await?;
        }
        Ok(())
    }
}

impl From<CoverDownloadWithManga> for MangaDownload {
    fn from(value: CoverDownloadWithManga) -> Self {
        Self {
            dirs_options: value.dirs_options,
            http_client: value.http_client,
            manga_id: value.manga_id,
        }
    }
}

impl<'a> From<&'a CoverDownloadWithManga> for MangaDownload {
    fn from(value: &'a CoverDownloadWithManga) -> Self {
        Self {
            dirs_options: value.dirs_options.clone(),
            http_client: value.http_client.clone(),
            manga_id: value.manga_id,
        }
    }
}

#[async_trait::async_trait]
pub trait AccessMangaDownload: AccessDownloadTasks + AccessHistory + Sized + Send + Sync {
    async fn download<'a>(
        &'a mut self,
        manga_download: &'a MangaDownload,
    ) -> ManagerCoreResult<()> {
        manga_download.download_manga(self).await
    }
}
