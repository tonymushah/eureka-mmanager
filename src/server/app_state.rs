use crate::core::ManagerCoreResult;
use crate::download::DownloadTaks;
use crate::r#static::history::HistoryMap;
use crate::settings::file_history::{HistoryWFile, HistoryEntry};
use crate::settings::files_dirs::DirsOptions;
use crate::settings::server_options::ServerOptions;
use crate::verify_all_fs;
use futures::Future;
use futures::lock::Mutex;
use mangadex_api::{HttpClient, HttpClientRef, MangaDexClient};
use mangadex_api_types_rust::RelationshipType;
use reqwest::header::{HeaderMap, HeaderValue};
use reqwest::Client;
use tokio::task::AbortHandle;

use super::traits::{AccessHistory, AccessDownloadTasks};

#[derive(Clone)]
pub struct AppState {
    pub http_client: HttpClientRef,
    pub dir_options: DirsOptions,
    pub server_options: ServerOptions,
    pub download_tasks: DownloadTaks,
    pub history: HistoryMap,
}

impl AppState {
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
            dir_options,
            server_options,
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
        Ok(HttpClientRef::new(Mutex::new(HttpClient::new(client))))
    }
    pub async fn load_dir_options_history() -> ManagerCoreResult<(DirsOptions, HistoryMap)> {
        let dir_options = DirsOptions::new()?;
        let history = HistoryMap::init(&dir_options, None).await?;
        Ok((dir_options, history))
    }
    pub async fn init() -> ManagerCoreResult<Self> {
        verify_all_fs()?;
        let http_client = Self::load_default_http_client()?;
        let (dir_options, history) = Self::load_dir_options_history().await?;
        let server_options = ServerOptions::new()?;
        Ok(Self {
            http_client,
            dir_options,
            server_options,
            download_tasks: Default::default(),
            history,
        })
    }
}

#[async_trait::async_trait]
impl AccessHistory for AppState {
    async fn init_history(
        &self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<()>{
        self.history.init_history(&self.dir_options, relationship_type).await
    }
    async fn get_history_w_file_by_rel(
        &self,
        relationship_type: RelationshipType,
    ) -> std::io::Result<HistoryWFile> {
        self.history.get_history_w_file_by_rel(relationship_type).await
    }
    async fn get_history_w_file_by_rel_or_init(
        &self,
        relationship_type: RelationshipType,
    ) -> ManagerCoreResult<HistoryWFile>{
        self.history.get_history_w_file_by_rel_or_init(relationship_type, &self.dir_options).await
    }
    async fn insert_in_history(
        &self,
        to_insert: &HistoryEntry,
    ) -> ManagerCoreResult<()> {
        self.history.insert_in_history(to_insert, &self.dir_options).await
    }
    async fn remove_in_history(
        &self,
        to_remove: &HistoryEntry,
    ) -> ManagerCoreResult<()>{
        self.history.remove_in_history(to_remove, &self.dir_options).await
    }
    async fn commit_rel(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()>{
        self.history.commit_rel(relationship_type).await
    }
    async fn rollback_rel(&self, relationship_type: RelationshipType) -> ManagerCoreResult<()>{
        self.history.rollback_rel(relationship_type).await
    }
}

#[async_trait::async_trait]
impl AccessDownloadTasks for AppState {
    async fn verify_limit(&self) -> bool{
        self.download_tasks.verify_limit().await
    }
    async fn spawn<F>(&mut self, task : F) -> ManagerCoreResult<AbortHandle> 
    where 
        F : Future<Output = ()> + Send + 'static
    {
        self.download_tasks.spawn(task).await
    }
    async fn lock_spawn<F>(&mut self, task : F) -> AbortHandle
    where 
        F : Future<Output = ()> + Send + 'static
    {
        self.download_tasks.lock_spawn(task).await
    }
    async fn spawn_with_data<T>(&mut self, task : T) -> ManagerCoreResult<T::Output> 
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static
    {
        self.download_tasks.spawn_with_data(task).await
    }
    async fn lock_spawn_with_data<T>(&mut self, task : T) -> ManagerCoreResult<T::Output> 
    where
        T: Future + Send + 'static,
        T::Output: Send + 'static
    {
        self.download_tasks.lock_spawn_with_data(task).await
    }
}