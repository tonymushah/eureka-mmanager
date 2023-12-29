use std::io::Write;
use std::sync::Arc;

use mangadex_api::{HttpClientRef, MangaDexClient};
use mangadex_api_schema_rust::v5::MangaData;
use mangadex_api_types_rust::ReferenceExpansionResource;
use uuid::Uuid;

use crate::core::ManagerCoreResult;
use crate::server::traits::{AccessDownloadTasks, AccessHistory};
use crate::settings::files_dirs::DirsOptions;
use crate::utils::manga::MangaUtilsWithMangaId;
use crate::utils::ExtractData;

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
    pub async fn download_manga<'a, D>(
        &'a self,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<MangaData>
    where
        D: AccessDownloadTasks,
    {
        let manga_utils: MangaUtilsWithMangaId = From::from(self);
        let manga_utils_move = manga_utils.clone();
        let id = self.manga_id;
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let task_res: MangaData = task_manager
            .lock_spawn_with_data(async move {
                let manga = client
                    .manga()
                    .id(id)
                    .get()
                    .include(ReferenceExpansionResource::Manga)
                    .include(ReferenceExpansionResource::CoverArt)
                    .include(ReferenceExpansionResource::Author)
                    .include(ReferenceExpansionResource::Artist)
                    .include(ReferenceExpansionResource::Tag)
                    .include(ReferenceExpansionResource::Creator)
                    .send()
                    .await?;
                let mut writer = manga_utils_move.get_buf_writer()?;
                serde_json::to_writer(&mut writer, &manga)?;
                writer.flush()?;
                Ok::<MangaData, crate::Error>(manga)
            })
            .await??;
        let cover_download: CoverDownloadWithManga = From::from(self);
        if let Ok(is_there) = manga_utils.is_cover_there() {
            if !is_there {
                cover_download.download(task_manager).await?;
            }
        } else {
            cover_download.download(task_manager).await?;
        }
        Ok(task_res)
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
    ) -> ManagerCoreResult<MangaData> {
        manga_download.download_manga(self).await
    }
}
