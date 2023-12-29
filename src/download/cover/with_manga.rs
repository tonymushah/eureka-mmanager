use std::{
    fs::File,
    io::{BufWriter, Write},
    sync::Arc,
};

use mangadex_api::{utils::download::cover::CoverQuality, HttpClientRef, MangaDexClient};
use mangadex_api_schema_rust::v5::CoverData;
use mangadex_api_types_rust::RelationshipType;
use uuid::Uuid;

use crate::{
    download::manga::MangaDownload,
    server::traits::{AccessDownloadTasks, AccessHistory},
    settings::files_dirs::DirsOptions,
    utils::manga::MangaUtilsWithMangaId,
    ManagerCoreResult,
};

use super::CoverDownload;

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
    pub async fn download<'a, D>(&self, task_manager: &'a mut D) -> ManagerCoreResult<CoverData>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let manga = client.manga().id(self.manga_id).get().send().await?;
        let cover_id = manga
            .data
            .relationships
            .iter()
            .find(|related| related.type_ == RelationshipType::CoverArt)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no cover art found for manga {}", self.manga_id),
            ))?
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
    ) -> ManagerCoreResult<CoverData>
    where
        D: AccessDownloadTasks,
    {
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let manga = client.manga().id(self.manga_id).get().send().await?;
        let cover_id = manga
            .data
            .relationships
            .iter()
            .find(|related| related.type_ == RelationshipType::CoverArt)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no cover art found for manga {}", self.manga_id),
            ))?
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
            .get()
            .add_manga_id(self.manga_id)
            .limit(limit)
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
        {
            let files = File::create(format!("covers/lists/{}.json", self.manga_id))?;
            let mut writer = BufWriter::new(files);
            writer.write_all(jsons.to_string().as_bytes())?;
            writer.flush()?;
        }
        Ok(jsons)
    }
}

impl From<MangaUtilsWithMangaId> for CoverDownloadWithManga {
    fn from(value: MangaUtilsWithMangaId) -> Self {
        Self {
            dirs_options: value.manga_utils.dirs_options,
            http_client: value.manga_utils.http_client_ref,
            manga_id: value.manga_id,
        }
    }
}

impl<'a> From<&'a MangaUtilsWithMangaId> for CoverDownloadWithManga {
    fn from(value: &'a MangaUtilsWithMangaId) -> Self {
        Self {
            dirs_options: value.manga_utils.dirs_options.clone(),
            http_client: value.manga_utils.http_client_ref.clone(),
            manga_id: value.manga_id,
        }
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
    ) -> ManagerCoreResult<CoverData> {
        cover_download.download(self).await
    }
    async fn download_with_quality<'a>(
        &'a mut self,
        cover_download: &'a CoverDownloadWithManga,
        quality: CoverQuality,
    ) -> ManagerCoreResult<CoverData> {
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
