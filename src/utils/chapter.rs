use log::info;
use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::{v5::ChapterAttributes, ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
use std::{
    fs::File,
    io::{BufReader, ErrorKind},
    path::Path,
    sync::Arc,
};
use tokio_stream::{Stream, StreamExt};

use crate::{
    core::{Error, ManagerCoreResult},
    download::chapter::ChapterDownload,
    download::manga::MangaDownload,
    methods::get::_find_all_downloaded_chapter::GetChapterQuery,
    server::traits::{AccessDownloadTasks, AccessHistory},
    settings::{
        file_history::{
            history_w_file::traits::{
                NoLFAsyncAutoCommitRollbackInsert, NoLFAsyncAutoCommitRollbackRemove,
            },
            HistoryEntry, HistoryWFile,
        },
        files_dirs::DirsOptions,
    },
};

use self::get_all_chapter::AsyncGetAllChapter;

use super::{collection::Collection, cover::CoverUtils, manga::MangaUtils};

mod get_all_chapter;

#[derive(Clone)]
pub struct ChapterUtils {
    pub(crate) dirs_options: Arc<DirsOptions>,
    pub(crate) http_client_ref: HttpClientRef,
}

impl ChapterUtils {
    pub fn new(dirs_options: Arc<DirsOptions>, http_client_ref: HttpClientRef) -> Self {
        Self {
            dirs_options,
            http_client_ref,
        }
    }
    pub(self) fn is_chapter_manga_there(&self, chap_id: String) -> ManagerCoreResult<bool> {
        let manga_utils: MangaUtils = From::from(self);
        if !chap_id.is_empty() {
            let path = self
                .dirs_options
                .chapters_add(format!("{}/data.json", chap_id).as_str());
            let chap_data: ApiData<ApiObject<ChapterAttributes>> =
                serde_json::from_reader(BufReader::new(File::open(path)?))?;
            let manga_id: uuid::Uuid = match chap_data
                .data
                .relationships
                .iter()
                .find(|rel| rel.type_ == RelationshipType::Manga)
            {
                Some(data) => data.id,
                None => {
                    return ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Seems like your chapter has no manga related to him",
                    )));
                }
            };
            Ok(manga_utils.with_id(format!("{}", manga_id)).is_there()?)
        } else {
            ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "the chap_id should'nt be empty",
            )))
        }
    }
    pub(self) async fn update_chap_by_id<'a, H, D>(
        &'a self,
        id: String,
        history: &'a mut H,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<ApiData<ApiObject<ChapterAttributes>>>
    where
        H: AccessHistory,
        D: AccessDownloadTasks,
    {
        let chap_id = uuid::Uuid::parse_str(id.as_str())?;
        let entry = HistoryEntry::new(chap_id, RelationshipType::Chapter);
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackInsert<HistoryEntry>>::insert(
            history, entry,
        )
        .await?;
        let data = ChapterDownload::new(
            chap_id,
            self.dirs_options.clone(),
            self.http_client_ref.clone(),
        )
        .download_json_data(task_manager)
        .await?;
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackRemove<HistoryEntry>>::remove(
            history, entry,
        )
        .await?;
        Ok(data)
    }
    pub(self) async fn patch_manga_by_chapter<'a, H, D>(
        &'a self,
        chap_id: String,
        history: &'a mut H,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        H: AccessHistory,
        D: AccessDownloadTasks,
    {
        let manga_utils: MangaUtils = From::from(self);
        let chapter: ApiObject<ChapterAttributes> = self.get_chapter_by_id(chap_id.clone())?;
        let manga = match chapter
            .relationships
            .iter()
            .find(|related| related.type_ == RelationshipType::Manga)
        {
            None => {
                return Err(Error::Io(std::io::Error::new(
                    ErrorKind::Other,
                    format!("can't find manga in the chapter {}", chap_id.clone()).as_str(),
                )));
            }
            Some(data) => data,
        };
        let manga_id = manga.id;
        let type_ = manga.type_;
        let history_entry = HistoryEntry::new(manga_id, type_);
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackInsert<HistoryEntry>>::insert(
            history,
            history_entry,
        )
        .await?;
        MangaDownload::new(
            manga_id,
            manga_utils.dirs_options,
            manga_utils.http_client_ref,
        )
        .download_manga(task_manager)
        .await?;
        let jsons = serde_json::json!({
            "result" : "ok",
            "type" : "manga",
            "id" : manga_id.hyphenated()
        });
        info!("downloaded {}.json", manga_id.hyphenated());
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackRemove<HistoryEntry>>::remove(
            history,
            history_entry,
        )
        .await?;
        Ok(jsons)
    }
    pub(self) fn get_chapter_by_id<T>(
        &self,
        chap_id: T,
    ) -> ManagerCoreResult<ApiObject<ChapterAttributes>>
    where
        T: ToString,
    {
        let path = self
            .dirs_options
            .chapters_add(format!("{}/data.json", chap_id.to_string()).as_str());
        let data: ApiData<ApiObject<ChapterAttributes>> =
            serde_json::from_reader(BufReader::new(File::open(path)?))?;
        Ok(data.data)
    }
    pub fn get_chapters_by_stream_id<'a, T>(
        &'a self,
        chap_ids: T,
    ) -> impl Stream<Item = ApiObject<ChapterAttributes>> + 'a
    where
        T: Stream<Item = String> + std::marker::Unpin + 'a,
    {
        chap_ids.filter_map(|id| {
            if let Ok(data_) = self.get_chapter_by_id(id) {
                Some(data_)
            } else {
                None
            }
        })
    }
    pub fn get_all_chapters_data(
        &self,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + '_> {
        Ok(self.get_chapters_by_stream_id(Box::pin(self.get_all_chapter_without_history()?)))
    }
    pub fn get_chapters_by_vec_id(
        &self,
        chap_ids: Vec<String>,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + '_> {
        Ok(tokio_stream::iter(chap_ids).filter_map(move |id| {
            if let Ok(data_) = self.get_chapter_by_id(id) {
                Some(data_)
            } else {
                None
            }
        }))
    }
    pub fn get_all_chapter_without_history(
        &self,
    ) -> ManagerCoreResult<impl Stream<Item = String> + '_> {
        let file_dirs = self.dirs_options.clone();
        let path = file_dirs.chapters_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = std::fs::read_dir(path.as_str())?;
            Ok(
                tokio_stream::iter(list_dir.flatten()).filter_map(move |files| {
                    if let Some(data) = files.file_name().to_str() {
                        if Path::new(format!("{}/data.json", file_dirs.chapters_add(data)).as_str())
                            .exists()
                        {
                            Some(data.to_string())
                        } else {
                            None
                        }
                    } else {
                        None
                    }
                }),
            )
        } else {
            Err(Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't find the chapter directory",
            )))
        }
    }
    pub async fn get_all_chapter<'a, H>(
        &'a self,
        parameters: Option<GetAllChapter>,
        history: &'a mut H,
    ) -> ManagerCoreResult<impl Stream<Item = String> + 'a>
    where
        H: AccessHistory,
    {
        let file_dirs = self.dirs_options.clone();
        let all_chapters = Box::pin(self.get_all_chapter_without_history()?);
        let parameters = parameters.unwrap_or_default();
        let hist: HistoryWFile = history
            .get_history_w_file_by_rel_or_init(RelationshipType::Chapter)
            .await?;
        let h = hist.owned_read_history()?;
        let re_h = h.clone();
        Ok(AsyncGetAllChapter::new(
            parameters,
            h,
            all_chapters,
            tokio_stream::iter(re_h).filter(move |entry| {
                Path::new(
                    format!(
                        "{}/data.json",
                        file_dirs.chapters_add(entry.to_string().as_str())
                    )
                    .as_str(),
                )
                .exists()
            }),
        ))
    }
    pub async fn get_all_downloaded_chapters<'a, H>(
        &'a self,
        parameters: Option<GetChapterQuery>,
        history: &'a mut H,
    ) -> ManagerCoreResult<Collection<String>>
    where
        H: AccessHistory,
    {
        if let Some(param) = parameters {
            let stream = self
                .get_all_chapter(Some(GetAllChapter::from(param.clone())), history)
                .await?;
            let collection: Collection<String> = Collection::from_async_stream(
                stream,
                param.clone().limit.unwrap_or(10),
                param.offset.unwrap_or(0),
            )
            .await?;
            Ok(collection)
        } else {
            let stream = self.get_all_chapter(None, history).await?;
            let collection: Collection<String> =
                Collection::from_async_stream(stream, 10, 0).await?;
            Ok(collection)
        }
    }
    pub fn with_id(&self, chapter_id: String) -> ChapterUtilsWithID {
        ChapterUtilsWithID {
            chapter_utils: self.clone(),
            chapter_id,
        }
    }
}

#[derive(Clone)]
pub struct ChapterUtilsWithID {
    pub chapter_utils: ChapterUtils,
    pub(crate) chapter_id: String,
}

impl ChapterUtilsWithID {
    pub fn new(chapter_id: String, chapter_utils: ChapterUtils) -> Self {
        Self {
            chapter_utils,
            chapter_id,
        }
    }
    pub fn is_manga_there(&self) -> ManagerCoreResult<bool> {
        self.chapter_utils
            .is_chapter_manga_there(self.chapter_id.clone())
    }
    pub async fn update<'a, H, D>(
        &'a self,
        history: &'a mut H,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<ApiData<ApiObject<ChapterAttributes>>>
    where
        H: AccessHistory,
        D: AccessDownloadTasks,
    {
        self.chapter_utils
            .update_chap_by_id(self.chapter_id.clone(), history, task_manager)
            .await
    }
    pub async fn patch_manga<'a, H, D>(
        &'a self,
        history: &'a mut H,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<serde_json::Value>
    where
        H: AccessHistory,
        D: AccessDownloadTasks,
    {
        self.chapter_utils
            .patch_manga_by_chapter(self.chapter_id.clone(), history, task_manager)
            .await
    }
    pub fn get_chapter(&self) -> ManagerCoreResult<ApiObject<ChapterAttributes>> {
        self.chapter_utils
            .get_chapter_by_id(self.chapter_id.clone())
    }
}

#[async_trait::async_trait]
pub trait AccessChapterUtisWithID:
    AccessDownloadTasks + AccessHistory + Sized + Send + Sync + Clone
{
    async fn update<'a>(
        &'a mut self,
        chapter_util_with_id: &'a ChapterUtilsWithID,
    ) -> ManagerCoreResult<ApiData<ApiObject<ChapterAttributes>>> {
        let mut reself = self.clone();
        chapter_util_with_id.update(self, &mut reself).await
    }
    async fn patch_manga<'a>(
        &'a mut self,
        chapter_util_with_id: &'a ChapterUtilsWithID,
    ) -> ManagerCoreResult<serde_json::Value> {
        let mut reself = self.clone();
        chapter_util_with_id.patch_manga(self, &mut reself).await
    }
}

impl From<MangaUtils> for ChapterUtils {
    fn from(value: MangaUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a MangaUtils> for ChapterUtils {
    fn from(value: &'a MangaUtils) -> Self {
        Self::new(value.dirs_options.clone(), value.http_client_ref.clone())
    }
}

impl From<CoverUtils> for ChapterUtils {
    fn from(value: CoverUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a CoverUtils> for ChapterUtils {
    fn from(value: &'a CoverUtils) -> Self {
        Self::new(value.dirs_options.clone(), value.http_client_ref.clone())
    }
}

impl From<ChapterDownload> for ChapterUtils {
    fn from(value: ChapterDownload) -> Self {
        Self {
            dirs_options: value.dirs_options,
            http_client_ref: value.http_client,
        }
    }
}

impl<'a> From<&'a ChapterDownload> for ChapterUtils {
    fn from(value: &'a ChapterDownload) -> Self {
        Self {
            dirs_options: value.dirs_options.clone(),
            http_client_ref: value.http_client.clone(),
        }
    }
}

impl<'a> From<&'a mut ChapterDownload> for ChapterUtils {
    fn from(value: &'a mut ChapterDownload) -> Self {
        Self {
            dirs_options: value.dirs_options.clone(),
            http_client_ref: value.http_client.clone(),
        }
    }
}

impl From<ChapterDownload> for ChapterUtilsWithID {
    fn from(value: ChapterDownload) -> Self {
        let chapter_id = value.chapter_id.to_string();
        Self {
            chapter_utils: From::from(value),
            chapter_id,
        }
    }
}

impl From<&ChapterDownload> for ChapterUtilsWithID {
    fn from(value: &ChapterDownload) -> Self {
        Self {
            chapter_utils: From::from(value),
            chapter_id: value.chapter_id.to_string(),
        }
    }
}

#[derive(Clone)]
pub struct GetAllChapter {
    pub include_fails: bool,
    pub only_fails: bool,
}

impl Default for GetAllChapter {
    fn default() -> Self {
        Self {
            include_fails: true,
            only_fails: false,
        }
    }
}

impl From<GetChapterQuery> for GetAllChapter {
    fn from(value: GetChapterQuery) -> Self {
        Self {
            include_fails: value.include_fails.unwrap_or(true),
            only_fails: value.only_fails.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    #[tokio::test]
    pub async fn test_get_chapter_by_id() {
        let data = ChapterUtils::new(
            Arc::new(DirsOptions::new().unwrap()),
            HttpClientRef::default(),
        )
        .get_chapter_by_id("167fb8f3-1180-4b1c-ac02-a01dc24b8865".to_string())
        .unwrap();
        println!("{}", serde_json::to_string(&data).unwrap());
    }
    #[tokio::test]
    pub async fn test_get_chapters_by_vec_ids() {
        let dir_options = Arc::new(DirsOptions::new().unwrap());
        let client = HttpClientRef::default();
        let chapter_utils = ChapterUtils::new(dir_options, client);
        let manga_utils: MangaUtils = From::from(chapter_utils.clone());
        let manga_id = "17727b0f-c9f2-4ab5-a0b1-b7b0cf6c1fc8".to_string();
        let this_manga_utils = manga_utils.with_id(manga_id);
        let manga_downloads = Box::pin(this_manga_utils.find_all_downloades().unwrap());
        let datas = chapter_utils.get_chapters_by_stream_id(manga_downloads);
        tokio::pin!(datas);
        while let Some(chap) = datas.next().await {
            println!("{}", serde_json::to_string(&chap).unwrap());
        }
    }
}
