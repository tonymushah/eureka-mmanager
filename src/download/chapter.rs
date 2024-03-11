// Imports used for downloading the pages to a file.
// They are not used because we're just printing the raw bytes.

mod download;
mod download_chapter;
mod download_chapter_data_saver;
mod download_json_data;
mod end_transation;
mod start_transation;
mod verify_chapter_and_manga;

#[derive(Debug, Deserialize, Serialize)]
pub struct DownloadChapterResult {
    pub result: ResultType,
    pub dir: String,
    pub downloaded: Vec<String>,
    pub errors: Vec<String>,
}

use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::v5::ChapterAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::ResultType;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use crate::core::ManagerCoreResult;
use crate::server::traits::{AccessDownloadTasks, AccessHistory};
use crate::settings::files_dirs::DirsOptions;
use crate::utils::chapter::ChapterUtilsWithID;

#[derive(Clone)]
pub struct ChapterDownload {
    pub dirs_options: Arc<DirsOptions>,
    pub http_client: HttpClientRef,
    pub chapter_id: Uuid,
}

impl ChapterDownload {
    pub fn new(
        chapter_id: Uuid,
        dirs_options: Arc<DirsOptions>,
        http_client: HttpClientRef,
    ) -> Self {
        Self {
            dirs_options,
            http_client,
            chapter_id,
        }
    }
}

impl From<ChapterUtilsWithID> for ChapterDownload {
    fn from(value: ChapterUtilsWithID) -> Self {
        Self {
            dirs_options: value.chapter_utils.dirs_options,
            http_client: value.chapter_utils.http_client_ref,
            chapter_id: value.chapter_id,
        }
    }
}

impl From<&ChapterUtilsWithID> for ChapterDownload {
    fn from(value: &ChapterUtilsWithID) -> Self {
        Self {
            dirs_options: value.chapter_utils.dirs_options.clone(),
            http_client: value.chapter_utils.http_client_ref.clone(),
            chapter_id: value.chapter_id,
        }
    }
}

#[async_trait::async_trait]
pub trait AccessChapterDownload: AccessDownloadTasks + AccessHistory + Sized + Send + Sync {
    async fn download_json_data<'a>(
        &'a mut self,
        chapter_download: &'a ChapterDownload,
    ) -> ManagerCoreResult<ApiData<ApiObject<ChapterAttributes>>> {
        chapter_download.download_json_data(self).await
    }
    async fn download<'a>(
        &'a mut self,
        chapter_download: &'a ChapterDownload,
    ) -> ManagerCoreResult<DownloadChapterResult> {
        chapter_download.download_chapter(self).await
    }
    async fn download_data_saver<'a>(
        &'a mut self,
        chapter_download: &'a ChapterDownload,
    ) -> ManagerCoreResult<DownloadChapterResult> {
        chapter_download.download_chapter_data_saver(self).await
    }
}

#[cfg(test)]
mod tests {
    use crate::server::AppState;

    use super::*;

    /// this will test the downloading for this chapter
    /// https://mangadex.org/chapter/b8e7925e-581a-4c06-a964-0d822053391a
    ///
    /// Dev note : Don't go there it's an H...
    #[tokio::test]
    async fn test_download_chapter_normal() {
        let mut app_state = AppState::init().await.unwrap();
        let chapter_id = "b8e7925e-581a-4c06-a964-0d822053391a";
        let chapter_download = app_state.chapter_download(Uuid::parse_str(chapter_id).unwrap());
        <AppState as AccessChapterDownload>::download(&mut app_state, &chapter_download)
            .await
            .unwrap();
    }
}
