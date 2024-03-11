use std::{
    fs::File,
    io::{BufWriter, Write},
};

use log::info;
use mangadex_api::{utils::download::chapter::DownloadMode, MangaDexClient};
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::{server::traits::AccessDownloadTasks, ManagerCoreResult};

use super::ChapterDownload;

impl ChapterDownload {
    pub(crate) async fn download<T>(
        ctx: &mut T,
        client: MangaDexClient,
        chapter_id: Uuid,
        mode: DownloadMode,
        chapter_dir: String,
    ) -> ManagerCoreResult<(Vec<String>, Vec<String>, bool, String)>
    where
        T: AccessDownloadTasks,
    {
        ctx.lock_spawn_with_data(async move {
            let mut files_: Vec<String> = Vec::new();

            let mut errors: Vec<String> = Vec::new();
            let mut has_error = false;
            let stream = client
                .download()
                .chapter(chapter_id)
                .report(true)
                .mode(mode)
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
                    match File::create(format!("{}/{}", chapter_dir.clone(), filename.clone())) {
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
        .await?
    }
}
