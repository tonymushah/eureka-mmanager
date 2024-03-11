// Imports used for downloading the pages to a file.
// They are not used because we're just printing the raw bytes.

mod download_json_data;
mod end_transation;
mod start_transation;
mod verify_chapter_and_manga;

use log::info;
use mangadex_api::{utils::download::chapter::DownloadMode, v5::MangaDexClient, HttpClientRef};
use mangadex_api_schema_rust::v5::ChapterAttributes;
use mangadex_api_schema_rust::{ApiData, ApiObject};
use serde_json::json;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::sync::Arc;
use tokio_stream::StreamExt;
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

    pub async fn download_chapter<'a, T>(
        &'a self,
        ctx: &'a mut T,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        T: AccessHistory + AccessDownloadTasks,
    {
        let history_entry = self.start_transation(ctx).await?;
        let chapter_id = history_entry.get_id();

        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let files_dirs = self.dirs_options.clone();
        let chapter_top_dir = files_dirs.chapters_add(chapter_id.hyphenated().to_string().as_str());
        let chapter_dir = format!("{}/data", chapter_top_dir);

        std::fs::create_dir_all(&chapter_dir)?;

        info!("chapter dir created");

        self.verify_chapter_and_manga(ctx).await?;

        let task: ManagerCoreResult<(Vec<String>, Vec<String>, bool, String)> = ctx
            .lock_spawn_with_data(async move {
                let mut files_: Vec<String> = Vec::new();

                let mut errors: Vec<String> = Vec::new();
                let mut has_error = false;
                let stream = client
                    .download()
                    .chapter(chapter_id)
                    .report(true)
                    .mode(DownloadMode::Normal)
                    .build()?
                    .download_stream_with_checker(|filename, response| {
                        let pre_file = match File::open(format!(
                            "{}/{}",
                            chapter_dir.clone(),
                            filename.filename.clone()
                        )) {
                            Ok(d) => d,
                            Err(_) => return false,
                        };
                        let content_length = match response.content_length() {
                            None => return false,
                            Some(ctt_lgth) => ctt_lgth,
                        };
                        let pre_file_metadata = match pre_file.metadata() {
                            Ok(metadata) => metadata,
                            Err(_) => return false,
                        };
                        content_length == pre_file_metadata.len()
                    })
                    .await?;

                tokio::pin!(stream);

                while let Some(((filename, bytes_), index, len)) = stream.next().await {
                    if let Ok(bytes) = bytes_ {
                        match File::create(format!("{}/{}", chapter_dir.clone(), filename.clone()))
                        {
                            Ok(file) => {
                                let res = {
                                    let mut buf_writer = BufWriter::new(file);
                                    buf_writer
                                        .write_all(&bytes)
                                        .and_then(|_| buf_writer.flush())
                                };
                                match res {
                                    Ok(_) => {
                                        info!("{index} - {len} : Downloaded {filename}");
                                        files_.push(filename);
                                    }
                                    Err(e) => {
                                        log::error!("{index} - {len} : {}", e.to_string());
                                        errors.push(filename);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("{index} - {len} : {}", e.to_string());
                                errors.push(filename);
                            }
                        }
                    } else if let Err(error) = bytes_ {
                        if let mangadex_api_types_rust::error::Error::SkippedDownload(f) = error {
                            info!("{index} - {len} : Skipped {}", f);
                        } else {
                            log::error!("{index} - {len} : {}", error.to_string());
                            errors.push(filename);
                        }
                    }
                }

                if !errors.is_empty() {
                    has_error = true;
                }
                Ok((files_, errors, has_error, chapter_dir.clone()))
            })
            .await?;

        let (files_, errors, has_error, chapter_dir) = task?;

        let jsons = json!({
            "result" : "ok",
            "dir" : chapter_dir,
            "downloaded" : files_,
            "errors" : errors
        });

        let file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(jsons.to_string().as_bytes())?;
        writer.flush()?;
        if !has_error {
            self.end_transation(history_entry, ctx).await?;
        }

        Ok(jsons)
    }

    pub async fn download_chapter_data_saver<'a, T>(
        &'a self,
        ctx: &'a mut T,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        T: AccessHistory + AccessDownloadTasks,
    {
        let history_entry = self.start_transation(ctx).await?;
        let chapter_id = history_entry.get_id();

        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let files_dirs = self.dirs_options.clone();
        let chapter_top_dir = files_dirs.chapters_add(chapter_id.hyphenated().to_string().as_str());
        let chapter_dir = format!("{}/data-saver", chapter_top_dir);

        std::fs::create_dir_all(&chapter_dir)?;

        info!("chapter dir created");

        self.verify_chapter_and_manga(ctx).await?;

        let task: ManagerCoreResult<(Vec<String>, Vec<String>, bool, String)> = ctx
            .lock_spawn_with_data(async move {
                let mut files_: Vec<String> = Vec::new();

                let mut errors: Vec<String> = Vec::new();
                let mut has_error = false;
                let stream = client
                    .download()
                    .chapter(chapter_id)
                    .report(true)
                    .mode(DownloadMode::DataSaver)
                    .build()?
                    .download_stream_with_checker(|filename, response| {
                        let pre_file = match File::open(format!(
                            "{}/{}",
                            chapter_dir.clone(),
                            filename.filename.clone()
                        )) {
                            Ok(d) => d,
                            Err(_) => return false,
                        };
                        let content_length = match response.content_length() {
                            None => return false,
                            Some(ctt_lgth) => ctt_lgth,
                        };
                        let pre_file_metadata = match pre_file.metadata() {
                            Ok(metadata) => metadata,
                            Err(_) => return false,
                        };
                        content_length == pre_file_metadata.len()
                    })
                    .await?;

                tokio::pin!(stream);

                while let Some(((filename, bytes_), index, len)) = stream.next().await {
                    if let Ok(bytes) = bytes_ {
                        match File::create(format!("{}/{}", chapter_dir.clone(), filename.clone()))
                        {
                            Ok(file) => {
                                let res = {
                                    let mut buf_writer = BufWriter::new(file);
                                    buf_writer
                                        .write_all(&bytes)
                                        .and_then(|_| buf_writer.flush())
                                };
                                match res {
                                    Ok(_) => {
                                        info!("{index} - {len} : Downloaded {filename}");
                                        files_.push(filename);
                                    }
                                    Err(e) => {
                                        log::error!("{index} - {len} : {}", e.to_string());
                                        errors.push(filename);
                                    }
                                }
                            }
                            Err(e) => {
                                log::error!("{index} - {len} : {}", e.to_string());
                                errors.push(filename);
                            }
                        }
                    } else if let Err(error) = bytes_ {
                        if let mangadex_api_types_rust::error::Error::SkippedDownload(f) = error {
                            info!("{index} - {len} : Skipped {}", f);
                        } else {
                            log::error!("{index} - {len} : {}", error.to_string());
                            errors.push(filename);
                        }
                    }
                }
                if !errors.is_empty() {
                    has_error = true;
                }
                Ok((files_, errors, has_error, chapter_dir.clone()))
            })
            .await?;
        let (files_, errors, has_error, chapter_dir) = task?;
        let jsons = json!({
            "result" : "ok",
            "dir" : chapter_dir,
            "downloaded" : files_,
            "errors" : errors
        });
        let file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(jsons.to_string().as_bytes())?;
        writer.flush()?;
        if !has_error {
            self.end_transation(history_entry, ctx).await?;
        }

        Ok(jsons)
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
    ) -> ManagerCoreResult<serde_json::Value> {
        chapter_download.download_chapter(self).await
    }
    async fn download_data_saver<'a>(
        &'a mut self,
        chapter_download: &'a ChapterDownload,
    ) -> ManagerCoreResult<serde_json::Value> {
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
