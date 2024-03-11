mod app_state;
mod chapters;

use std::{
    fs::File,
    io::{BufWriter, Write},
    time::{Duration, Instant},
};

use mangadex_api::MangaDexClient;
use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_desktop_api2::{
    download::chapter::{AccessChapterDownload, DownloadChapterResult},
    AppState, ManagerCoreResult,
};
use serde::{Deserialize, Serialize};
use tokio::test;
use toml::{Table, Value};
use uuid::Uuid;

#[derive(Debug, Deserialize, Serialize)]
struct DownloadResultSuccessLogItem {
    id: Option<Uuid>,
    downloaded: usize,
    errors: usize,
    acc: usize,
}

impl From<DownloadChapterResult> for DownloadResultSuccessLogItem {
    fn from(value: DownloadChapterResult) -> Self {
        let acc = value.downloaded.len() / (value.errors.len() + value.downloaded.len());
        Self {
            id: None,
            downloaded: value.downloaded.len(),
            errors: value.errors.len(),
            acc,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Default)]
struct DownloadResult {
    succes_log: Vec<DownloadResultSuccessLogItem>,

    failed: usize,
    success: usize,
    total: usize,
    acc: usize,
    download_time: Duration,
}

impl DownloadResult {
    fn increment_total(&mut self) {
        self.total += 1;
        self.compute_acc();
    }
    fn compute_acc(&mut self) {
        self.acc = self.success / self.total;
    }
    fn increment_success(&mut self) {
        self.success += 1;
        self.increment_total();
    }
    fn increment_failed(&mut self) {
        self.failed += 1;
        self.increment_total();
    }
    pub fn append(&mut self, id: Uuid, res: DownloadChapterResult) {
        let mut log: DownloadResultSuccessLogItem = res.into();
        log.id = Some(id);
        if log.errors == 0 {
            self.increment_success();
        } else {
            self.increment_failed();
        }
        self.succes_log.push(log);
    }
}

async fn download(
    chaps: &[ChapterObject],
    app: &mut AppState,
) -> ManagerCoreResult<DownloadResult> {
    let start = Instant::now();
    let mut res = DownloadResult::default();
    for chap in chaps {
        if let Ok(dres) = app.download(&app.chapter_download(chap.id)).await {
            res.append(chap.id, dres);
        } else {
            res.increment_failed();
        }
    }
    let finish = Instant::now();
    res.download_time = finish.duration_since(start);
    Ok(res)
}

fn show_chap_id(chaps: &[ChapterObject], returns: &mut Table) -> anyhow::Result<()> {
    let chap_ids = chaps.iter().map(|c| c.id).collect::<Vec<Uuid>>();
    returns.insert(String::from("ids"), Value::try_from(chap_ids)?);
    Ok(())
}

async fn run() -> anyhow::Result<toml::Table> {
    let mut returns = Table::new();
    let app = unsafe { app_state::get_mut().await? };

    let client = MangaDexClient::new_with_http_client_ref(app.http_client.clone());
    let chaps = chapters::get(&client).await?;

    show_chap_id(&chaps, &mut returns)?;

    let download_res = download(&chaps, app).await?;
    Table::try_from(download_res)?
        .into_iter()
        .for_each(|(k, v)| {
            returns.insert(k, v);
        });
    Ok(returns)
}

#[test]
async fn main() -> anyhow::Result<()> {
    let mut file = BufWriter::new(File::create(std::env::var("RES")?)?);
    writeln!(file, "## `download_chapter.rs` test results")?;

    match run().await {
        Ok(res) => {
            writeln!(file, "```toml")?;
            writeln!(file, "{}", toml::to_string_pretty(&res)?)?;
            writeln!(file, "```")?;
        }
        Err(e) => {
            writeln!(file, "> [!WARNING]")?;
            writeln!(file, "> {}", e)?;
        }
    }

    Ok(())
}
