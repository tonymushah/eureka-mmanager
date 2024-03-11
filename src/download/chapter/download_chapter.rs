use std::{
    fs::File,
    io::{BufWriter, Write},
};

use log::info;
use mangadex_api::{utils::download::chapter::DownloadMode, MangaDexClient};
use mangadex_api_types_rust::ResultType;

use crate::{
    server::traits::{AccessDownloadTasks, AccessHistory},
    ManagerCoreResult,
};

use super::{ChapterDownload, DownloadChapterResult};

impl ChapterDownload {
    pub async fn download_chapter<'a, T>(
        &'a self,
        ctx: &'a mut T,
    ) -> ManagerCoreResult<DownloadChapterResult>
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

        let (files_, errors, has_error, chapter_dir) = Self::download(
            ctx,
            client,
            chapter_id,
            DownloadMode::Normal,
            chapter_dir.clone(),
        )
        .await?;

        let jsons = DownloadChapterResult {
            result: ResultType::Ok,
            dir: chapter_dir.clone(),
            downloaded: files_,
            errors,
        };

        let file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
        let mut writer = BufWriter::new(file);
        writer.write_all(&serde_json::to_vec_pretty(&jsons)?)?;
        writer.flush()?;
        if !has_error {
            self.end_transation(history_entry, ctx).await?;
        }

        Ok(jsons)
    }
}
