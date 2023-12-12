use std::fmt::Debug;
use std::future::Future;
use std::sync::Arc;

use crate::core::ManagerCoreResult;
use crate::download::chapter::{AccessChapterDownload, ChapterDownload};
use crate::download::cover::{AccessCoverDownload, AccessCoverDownloadWithManga, CoverDownload};
use crate::download::manga::{AccessMangaDownload, MangaDownload};
use crate::download::DownloadTaks;
use crate::r#static::history::HistoryMap;
use crate::settings::file_history::history_w_file::traits::{
    AsyncAutoCommitRollbackInsert, AsyncAutoCommitRollbackRemove, AsyncCommitableWInput,
    AsyncRollBackableWInput, NoLFAsyncAutoCommitRollbackInsert, NoLFAsyncAutoCommitRollbackRemove,
};
use crate::settings::file_history::{
    AsyncInsert, AsyncIsIn, AsyncRemove, HistoryEntry, HistoryWFile, NoLFAsyncInsert,
    NoLFAsyncIsIn, NoLFAsyncRemove,
};
use crate::settings::files_dirs::DirsOptions;
use crate::settings::server_options::ServerOptions;
use crate::utils::chapter::{AccessChapterUtisWithID, ChapterUtils};
use crate::utils::cover::CoverUtils;
use crate::utils::manga::MangaUtils;
use crate::verify_all_fs;
#[cfg(feature = "actix_web")]
use actix_web::web::Data;
use mangadex_api::{HttpClient, HttpClientRef, MangaDexClient};
use mangadex_api_types_rust::RelationshipType;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use tokio::sync::RwLock;
use tokio::task::AbortHandle;

use super::traits::{AccessDownloadTasks, AccessHistory};

pub struct AppState {
    pub http_client: HttpClientRef,
    pub dir_options: Arc<DirsOptions>,
    #[cfg(feature = "actix_web")]
    pub server_options: Arc<ServerOptions>,
    pub download_tasks: DownloadTaks,
    pub history: HistoryMap,
}

impl Clone for AppState {
    fn clone(&self) -> Self {
        Self {
            http_client: self.http_client.clone(),
            dir_options: self.dir_options.clone(),
            #[cfg(feature = "actix_web")]
            server_options: self.server_options.clone(),
            download_tasks: self.download_tasks.clone(),
            history: self.history.clone(),
        }
    }
}

impl AppState {
    #[cfg(feature = "actix_web")]
    pub fn get_hostname_port(&self) -> (String, u16) {
        self.server_options.get_hostname_port()
    }
    pub async fn get_mangadex_client(&self) -> MangaDexClient {
        MangaDexClient::new_with_http_client_ref(self.http_client.clone())
    }
    pub fn new(
        http_client_ref: HttpClientRef,
        dir_options: DirsOptions,
        server_options: ServerOptions,
        download_tasks: DownloadTaks,
        history: HistoryMap,
    ) -> Self {
        Self {
            http_client: http_client_ref,
            dir_options: Arc::new(dir_options),
            #[cfg(feature = "actix_web")]
            server_options: Arc::new(server_options),
            download_tasks,
            history,
        }
    }
    pub fn load_default_http_client() -> ManagerCoreResult<HttpClientRef> {
        let mut headers = HeaderMap::new();
        headers.insert(
            "User-Agent",
            HeaderValue::from_static("special-eureka-manager/0.4.0"),
        );
        let client = Client::builder().default_headers(headers).build()?;
        Ok(HttpClientRef::new(RwLock::new(HttpClient::new(client))))
    }
    pub async fn load_dir_options_history() -> ManagerCoreResult<(DirsOptions, HistoryMap)> {
        let dir_options = DirsOptions::new()?;
        let history = HistoryMap::init(
            &dir_options,
            Some(vec![
                RelationshipType::Chapter,
                RelationshipType::Manga,
                RelationshipType::CoverArt,
            ]),
        )
        .await?;
        Ok((dir_options, history))
    }
    pub async fn init() -> ManagerCoreResult<Self> {
        verify_all_fs()?;
        let http_client = Self::load_default_http_client()?;
        let (dir_options, history) = Self::load_dir_options_history().await?;
        #[cfg(feature = "actix_web")]
        let server_options = ServerOptions::new()?;
        Ok(Self {
            http_client,
            dir_options: Arc::new(dir_options),
            #[cfg(feature = "actix_web")]
            server_options: Arc::new(server_options),
            download_tasks: Default::default(),
            history,
        })
    }
    pub fn chapter_utils(&self) -> ChapterUtils {
        ChapterUtils {
            dirs_options: self.dir_options.clone(),
            http_client_ref: self.http_client.clone(),
        }
    }
    pub fn manga_utils(&self) -> MangaUtils {
        MangaUtils {
            dirs_options: self.dir_options.clone(),
            http_client_ref: self.http_client.clone(),
        }
    }
    pub fn cover_utils(&self) -> CoverUtils {
        CoverUtils {
            dirs_options: self.dir_options.clone(),
            http_client_ref: self.http_client.clone(),
        }
    }
    pub fn manga_download(&self, id: uuid::Uuid) -> MangaDownload {
        MangaDownload {
            dirs_options: self.dir_options.clone(),
            http_client: self.http_client.clone(),
            manga_id: id,
        }
    }
    pub fn chapter_download(&self, id: uuid::Uuid) -> ChapterDownload {
        ChapterDownload {
            dirs_options: self.dir_options.clone(),
            http_client: self.http_client.clone(),
            chapter_id: id,
        }
    }
    pub fn cover_download(&self, id: uuid::Uuid) -> CoverDownload {
        CoverDownload {
            dirs_options: self.dir_options.clone(),
            http_client: self.http_client.clone(),
            cover_id: id,
        }
    }
}

#[async_trait::async_trait]
impl NoLFAsyncInsert<HistoryEntry> for AppState {
    type Output = <HistoryMap as NoLFAsyncInsert<(HistoryEntry, DirsOptions)>>::Output;
    async fn insert<'a>(&'a mut self, input: HistoryEntry) -> Self::Output {
        <HistoryMap as AsyncInsert<'a, (HistoryEntry, &'a DirsOptions)>>::insert(
            &mut self.history,
            (input, self.dir_options.as_ref()),
        )
        .await
    }
}

#[async_trait::async_trait]
impl NoLFAsyncRemove<HistoryEntry> for AppState {
    type Output = <HistoryMap as NoLFAsyncRemove<(HistoryEntry, DirsOptions)>>::Output;
    async fn remove<'a>(&'a mut self, input: HistoryEntry) -> Self::Output {
        <HistoryMap as AsyncRemove<'a, (HistoryEntry, &'a DirsOptions)>>::remove(
            &mut self.history,
            (input, self.dir_options.as_ref()),
        )
        .await
    }
}

#[async_trait::async_trait]
impl NoLFAsyncAutoCommitRollbackInsert<HistoryEntry> for AppState {
    type Output =
        <HistoryMap as NoLFAsyncAutoCommitRollbackInsert<(HistoryEntry, DirsOptions)>>::Output;
    async fn insert<'a>(&'a mut self, input: HistoryEntry) -> Self::Output {
        <HistoryMap as AsyncAutoCommitRollbackInsert<'a, (HistoryEntry, &'a DirsOptions)>>::insert(
            &mut self.history,
            (input, self.dir_options.as_ref()),
        )
        .await
    }
}

#[async_trait::async_trait]
impl NoLFAsyncAutoCommitRollbackRemove<HistoryEntry> for AppState {
    type Output =
        <HistoryMap as NoLFAsyncAutoCommitRollbackRemove<(HistoryEntry, DirsOptions)>>::Output;
    async fn remove<'a>(&'a mut self, input: HistoryEntry) -> Self::Output {
        <HistoryMap as AsyncAutoCommitRollbackRemove<'a, (HistoryEntry, &'a DirsOptions)>>::remove(
            &mut self.history,
            (input, self.dir_options.as_ref()),
        )
        .await
    }
}
#[async_trait::async_trait]
impl NoLFAsyncIsIn<HistoryEntry> for AppState {
    type Output = <HistoryMap as NoLFAsyncIsIn<(HistoryEntry, DirsOptions)>>::Output;
    async fn is_in<'a>(&'a self, to_use: HistoryEntry) -> Self::Output {
        <HistoryMap as AsyncIsIn<'a, (HistoryEntry, &'a DirsOptions)>>::is_in(
            &self.history,
            (to_use, self.dir_options.as_ref()),
        )
        .await
    }
}

#[async_trait::async_trait]
impl AccessHistory for AppState {
    async fn init_history(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()> {
        self.history
            .init_history(&self.dir_options, relationship_type)
            .await
    }
    async fn get_history_w_file_by_rel(
        &self,
        relationship_type: RelationshipType,
    ) -> std::io::Result<HistoryWFile> {
        self.history
            .get_history_w_file_by_rel(relationship_type)
            .await
    }
    async fn get_history_w_file_by_rel_or_init(
        &self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<HistoryWFile> {
        self.history
            .get_history_w_file_by_rel_or_init(relationship_type, &self.dir_options)
            .await
    }
    async fn commit_rel(&mut self, relationship_type: RelationshipType) -> ManagerCoreResult<()> {
        <HistoryMap as AsyncCommitableWInput<'life0, RelationshipType>>::commit(
            &mut self.history,
            relationship_type,
        )
        .await
    }
    async fn rollback_rel(&mut self, relationship_type: RelationshipType) -> ManagerCoreResult<()> {
        <HistoryMap as AsyncRollBackableWInput<'life0, RelationshipType>>::rollback(
            &mut self.history,
            relationship_type,
        )
        .await
    }
}

#[async_trait::async_trait]
impl AccessDownloadTasks for AppState {
    async fn verify_limit(&self) -> bool {
        self.download_tasks.verify_limit().await
    }
    async fn spawn<F>(&mut self, task: F) -> ManagerCoreResult<AbortHandle>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.download_tasks.spawn(task).await
    }
    async fn lock_spawn<F>(&mut self, task: F) -> ManagerCoreResult<AbortHandle>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.download_tasks.lock_spawn(task).await
    }
    async fn spawn_with_data<T>(&mut self, task: T) -> ManagerCoreResult<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + Debug + 'static,
    {
        self.download_tasks.spawn_with_data(task).await
    }
    async fn lock_spawn_with_data<T>(&mut self, task: T) -> ManagerCoreResult<T::Output>
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static,
    {
        self.download_tasks.lock_spawn_with_data(task).await
    }
}

#[async_trait::async_trait]
impl AccessChapterDownload for AppState {}

#[async_trait::async_trait]
impl AccessCoverDownload for AppState {}

#[async_trait::async_trait]
impl AccessMangaDownload for AppState {}

#[async_trait::async_trait]
impl AccessCoverDownloadWithManga for AppState {}

#[async_trait::async_trait]
impl AccessChapterUtisWithID for AppState {}

#[cfg(feature = "actix_web")]
impl From<Data<AppState>> for AppState {
    fn from(value: Data<AppState>) -> Self {
        let value = value.as_ref().clone();
        Self {
            http_client: value.http_client,
            dir_options: value.dir_options,
            server_options: value.server_options,
            download_tasks: value.download_tasks,
            history: value.history,
        }
    }
}
