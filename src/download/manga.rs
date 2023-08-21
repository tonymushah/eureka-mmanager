use std::fs::File;
use std::io::Write;

use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::v5::MangaAttributes;
use mangadex_api_schema_rust::{ApiObject, ApiData};
use uuid::Uuid;

use crate::core::ManagerCoreResult;
use crate::download::cover::cover_download_by_manga_id;
use crate::server::traits::AccessDownloadTasks;
use crate::settings::files_dirs::DirsOptions;
use crate::utils::send_request;

#[derive(Clone)]
pub struct MangaDownload {
    pub dirs_options: DirsOptions,
    pub http_client: HttpClientRef,
    pub manga_id: Uuid,
}

impl MangaDownload {
    pub fn new(manga_id: Uuid, dirs_options: DirsOptions, http_client: HttpClientRef) -> Self {
        Self {
            dirs_options,
            http_client,
            manga_id,
        }
    }
    /// download the manga with the specified id 
    pub async fn download_manga<'a, D>(&'a self, task_manager : &'a mut D) -> ManagerCoreResult<()>
        where 
            D : AccessDownloadTasks
    {
        let id = format!("{}", self.manga_id);
        let http_client = self.http_client.lock().await.client.clone();
        let task : ManagerCoreResult<String> = task_manager.lock_spawn_with_data(async move {
            let resp = send_request(http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, id)), 5).await?;
            let bytes = resp.bytes().await?;
            let bytes_string = String::from_utf8(bytes.to_vec())?;
            serde_json::from_str::<ApiData<ApiObject<MangaAttributes>>>(bytes_string.as_str())?;
            let mut file = (File::create(
                DirsOptions::new()?
                    .mangas_add(format!("{}.json", id).as_str())
            ))?;
            file.write_all(&bytes)?;
            Ok(id)
        }).await?;
        let id = task?;
        cover_download_by_manga_id(id.to_string().as_str(), self.http_client.clone()).await?;
        Ok(())
    }
}