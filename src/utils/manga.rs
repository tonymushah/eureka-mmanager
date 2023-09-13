use async_stream::stream;
use futures::Stream;
use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::v5::{ChapterAttributes, MangaAggregate, MangaAttributes};
use mangadex_api_schema_rust::{ApiData, ApiObject};
use mangadex_api_types_rust::{RelationshipType, ResultType};
use std::cmp::Ordering;
use std::fs::File;
use std::io::ErrorKind;
use std::path::Path;
use std::sync::Arc;
use tokio_stream::StreamExt;

use crate::core::ManagerCoreResult;
use crate::download::chapter::ChapterDownload;
use crate::download::manga::MangaDownload;
use crate::server::traits::AccessHistory;
use crate::settings::files_dirs::DirsOptions;

use super::chapter::ChapterUtils;
use super::collection::Collection;
use super::cover::CoverUtils;
use super::manga_aggregate::group_chapter_to_volume_aggregate;

#[derive(Clone)]
pub struct MangaUtils {
    pub(crate) dirs_options: Arc<DirsOptions>,
    pub(crate) http_client_ref: HttpClientRef,
}

impl<'a> MangaUtils {
    pub fn new(dirs_options: Arc<DirsOptions>, http_client_ref: HttpClientRef) -> Self {
        Self {
            dirs_options,
            http_client_ref,
        }
    }
    async fn is_chap_related_to_manga(
        &self,
        chap_id: String,
        manga_id: String,
    ) -> ManagerCoreResult<bool> {
        let chapter_utils: ChapterUtils = From::from(self);
        let chapter: ApiObject<ChapterAttributes> = chapter_utils.with_id(chap_id).get_chapter()?;
        let mut is = false;
        for relas in chapter.relationships {
            if relas.type_ == RelationshipType::Manga
                && relas.id.hyphenated().to_string() == manga_id
            {
                is = true;
            }
        }
        Ok(is)
    }
    pub(self) fn find_all_downloades_by_manga_id(
        &'a self,
        manga_id: String,
    ) -> impl Stream<Item = String> + 'a {
        stream! {
            let chapter_utils = <ChapterUtils as From<&'a Self>>::from(self);
            let stream_ = chapter_utils.get_all_chapter_without_history();
            if let Ok(stream) = stream_ {
                let mut stream = Box::pin(stream);
                while let Some(chap) = stream.next().await {
                    if let Ok(d) = self.is_chap_related_to_manga(chap.clone(), manga_id.clone()).await {
                        if d {
                            yield chap.clone();
                        }
                    };
                }
            }
        }
    }

    pub(self) async fn find_and_delete_all_downloades_by_manga_id(
        &'a self,
        manga_id: String,
    ) -> ManagerCoreResult<serde_json::Value> {
        let mut vecs: Vec<String> = Vec::new();
        let mut stream = Box::pin(self.find_all_downloades_by_manga_id(manga_id));
        while let Some(chapter_id) = stream.next().await {
            if std::fs::remove_dir_all(self.dirs_options.chapters_add(chapter_id.as_str())).is_ok()
            {
                vecs.push(chapter_id);
            }
        }
        Ok(serde_json::json!(vecs))
    }
    pub(self) fn get_downloaded_cover_of_a_manga<H>(
        &'a self,
        manga_id: String,
        _history: &'a mut H,
    ) -> ManagerCoreResult<impl Stream<Item = String> + 'a>
    where
        H: AccessHistory,
    {
        Ok(stream! {
            let cover_utils : CoverUtils = From::from(self);
            if let Ok(vecs) = cover_utils.get_all_cover(){
                let mut vecs = Box::pin(vecs);
                while let Some(data) = vecs.next().await {
                    let manga_id = manga_id.clone();
                    let data = data.clone();
                    let data_clone = data.clone();
                    if let core::result::Result::Ok(result) = self.is_cover_related_to_manga(manga_id, data) {
                        if result {
                            yield data_clone;
                        }
                    }
                }
            };
        })
    }
    pub(self) async fn get_downloaded_cover_of_a_manga_collection<H>(
        &'a self,
        manga_id: String,
        offset: usize,
        limit: usize,
        history: &'a mut H,
    ) -> ManagerCoreResult<Collection<String>>
    where
        H: AccessHistory,
    {
        let mut downloaded_covers =
            Box::pin(self.get_downloaded_cover_of_a_manga(manga_id, history)?);
        Collection::from_async_stream(&mut downloaded_covers, limit, offset).await
    }
    pub(self) fn is_manga_there(&self, manga_id: String) -> ManagerCoreResult<bool> {
        if !manga_id.is_empty() {
            let path = self
                .dirs_options
                .mangas_add(format!("{}.json", manga_id).as_str());
            ManagerCoreResult::Ok(Path::new(path.as_str()).exists())
        } else {
            ManagerCoreResult::Err(crate::core::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "the manga_id should'nt be empty",
            )))
        }
    }
    pub(self) fn get_manga_data_by_id(
        &self,
        manga_id: String,
    ) -> ManagerCoreResult<ApiObject<MangaAttributes>> {
        let path = self
            .dirs_options
            .mangas_add(format!("{}.json", manga_id).as_str());
        if Path::new(path.as_str()).exists() {
            let data: ApiData<ApiObject<MangaAttributes>> =
                serde_json::from_str(std::fs::read_to_string(path.as_str())?.as_str())?;
            Ok(data.data)
        } else {
            Err(crate::core::Error::Io(std::io::Error::new(
                ErrorKind::NotFound,
                format!("manga {} not found", manga_id),
            )))
        }
    }
    pub(self) fn is_manga_cover_there(&self, manga_id: String) -> Result<bool, std::io::Error> {
        if !manga_id.is_empty() {
            let path = self
                .dirs_options
                .mangas_add(format!("{}.json", manga_id).as_str());
            if !Path::new(path.as_str()).exists() {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "this manga hasn't been downloaded",
                ))
            } else {
                let manga_data: ApiData<ApiObject<MangaAttributes>> =
                    serde_json::from_reader(File::open(path)?)?;
                let cover_id: uuid::Uuid = match manga_data
                    .data
                    .relationships
                    .iter()
                    .find(|rel| rel.type_ == RelationshipType::CoverArt)
                {
                    None => {
                        return core::result::Result::Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            "this manga has no cover_art",
                        ))
                    }
                    Some(d) => d.id,
                };
                let cover_utils: CoverUtils = From::from(self);
                cover_utils.with_id(cover_id.to_string()).is_there()
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "the manga_id should'nt be empty",
            ))
        }
    }
    pub(self) fn is_cover_related_to_manga(
        &self,
        manga_id: String,
        cover_id: String,
    ) -> ManagerCoreResult<bool> {
        let cover_utils: CoverUtils = From::from(self);
        match cover_utils
            .with_id(cover_id)
            .get_data()?
            .data
            .relationships
            .iter()
            .find(|rel| rel.type_ == RelationshipType::Manga && rel.id.to_string() == manga_id)
        {
            None => Ok(false),
            Some(_) => self.is_manga_there(manga_id),
        }
    }
    pub fn get_manga_data_by_ids<T>(
        &'a self,
        mut manga_ids: T,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<MangaAttributes>> + 'a>
    where
        T: Stream<Item = String> + std::marker::Unpin + 'a,
    {
        Ok(stream! {
            while let Some(id) = manga_ids.next().await{
                if let Ok(data) = self.get_manga_data_by_id(id) {
                    yield data;
                }
            }
        })
    }
    pub fn get_manga_data_by_ids_old(
        &'a self,
        manga_ids: Vec<String>,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<MangaAttributes>> + 'a> {
        Ok(stream! {
            for id in manga_ids {
                if let Ok(data) = self.get_manga_data_by_id(id) {
                    yield data;
                }
            }
        })
    }
    pub fn get_all_downloaded_manga(
        &'a self,
    ) -> ManagerCoreResult<impl Stream<Item = String> + 'a> {
        let path = self.dirs_options.mangas_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = std::fs::read_dir(path.as_str())?.flatten();
            Ok(stream! {
                for file_ in list_dir {
                    if let Some(data) = file_.file_name().to_str() {
                        yield data.to_string().replace(".json", "")
                    }
                }
            })
        } else {
            Err(crate::core::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't find the manga directory",
            )))
        }
    }
    pub async fn get_downloaded_manga(
        &'a self,
        offset: usize,
        limit: usize,
        title__: Option<String>,
    ) -> ManagerCoreResult<Collection<String>> {
        let vecs = Box::pin(self.get_all_downloaded_manga()?);
        let manga_data = Box::pin(self.get_manga_data_by_ids(vecs)?);
        if let Some(title) = title__ {
            let mut data: Vec<String> = manga_data
                .filter(|data| {
                    for title_ in data.attributes.title.values() {
                        if title_
                            .to_lowercase()
                            .contains(title.to_lowercase().as_str())
                        {
                            return true;
                        }
                    }
                    for entry in &data.attributes.alt_titles {
                        for title_ in entry.values() {
                            if title_
                                .to_lowercase()
                                .contains(title.to_lowercase().as_str())
                            {
                                return true;
                            }
                        }
                    }
                    false
                })
                .map(|d| d.id.to_string())
                .collect()
                .await;
            Collection::new(&mut data, limit, offset)
        } else {
            let mut data: Vec<String> = manga_data.map(|d| d.id.to_string()).collect().await;
            Collection::new(&mut data, limit, offset)
        }
    }
    pub(self) async fn get_all_downloaded_chapter_data(
        &'a self,
        manga_id: String,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + 'a> {
        let chapter_utils: ChapterUtils = From::from(self);
        let data_ = Box::pin(self.find_all_downloades_by_manga_id(manga_id));
        Ok(stream! {
            let data_ = chapter_utils.get_chapters_by_stream_id(data_);
            let mut data = Box::pin(data_);
            while let Some(next) = data.next().await {
                yield next.clone()
            }
        })
        /*let mut data_vec: Vec<ApiObject<ChapterAttributes>> = data..await;
        data_vec.sort_by(|a, b| {
            let a = match a.attributes.chapter.clone() {
                None => return Ordering::Equal,
                Some(d) => d,
            };
            let b = match b.attributes.chapter.clone() {
                None => return Ordering::Equal,
                Some(d) => d,
            };
            let a_chp = match a.parse::<usize>() {
                core::result::Result::Ok(d) => d,
                Err(_) => return Ordering::Equal,
            };
            let b_chp = match b.parse::<usize>() {
                core::result::Result::Ok(d) => d,
                Err(_) => return Ordering::Equal,
            };
            a_chp.cmp(&b_chp)
        });
        core::result::Result::Ok(stream! {
            for chapter in data_vec {
                yield chapter
            }
        })*/
    }

    pub(self) async fn get_downloaded_chapter_of_a_manga(
        &'a self,
        manga_id: String,
        offset: usize,
        limit: usize,
    ) -> ManagerCoreResult<Collection<String>> {
        let all_downloaded = self.get_all_downloaded_chapter_data(manga_id).await?;
        let mut data = Box::pin(all_downloaded);
        let to_use: Collection<String> = Collection::from_async_stream(&mut data, limit, offset)
            .await?
            .convert_to(|d| d.id.to_string())?;
        Ok(to_use)
    }
    pub fn with_id(&self, manga_id: String) -> MangaUtilsWithMangaId {
        MangaUtilsWithMangaId {
            manga_utils: self.clone(),
            manga_id,
        }
    }
}

impl From<ChapterUtils> for MangaUtils {
    fn from(value: ChapterUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a ChapterUtils> for MangaUtils {
    fn from(value: &'a ChapterUtils) -> Self {
        Self::new(value.dirs_options.clone(), value.http_client_ref.clone())
    }
}

impl From<CoverUtils> for MangaUtils {
    fn from(value: CoverUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a CoverUtils> for MangaUtils {
    fn from(value: &'a CoverUtils) -> Self {
        Self::new(value.dirs_options.clone(), value.http_client_ref.clone())
    }
}

impl From<ChapterDownload> for MangaUtils {
    fn from(value: ChapterDownload) -> Self {
        Self {
            dirs_options: value.dirs_options,
            http_client_ref: value.http_client,
        }
    }
}

impl<'a> From<&'a ChapterDownload> for MangaUtils {
    fn from(value: &'a ChapterDownload) -> Self {
        Self {
            dirs_options: value.dirs_options.clone(),
            http_client_ref: value.http_client.clone(),
        }
    }
}

#[derive(Clone)]
pub struct MangaUtilsWithMangaId {
    pub(crate) manga_utils: MangaUtils,
    pub(crate) manga_id: String,
}

impl MangaUtilsWithMangaId {
    pub fn new(id: String, utils: MangaUtils) -> Self {
        Self {
            manga_utils: utils,
            manga_id: id,
        }
    }
    pub async fn is_chap_related(&self, chap_id: String) -> ManagerCoreResult<bool> {
        self.manga_utils
            .is_chap_related_to_manga(chap_id, self.manga_id.clone())
            .await
    }
    pub fn find_all_downloades(&self) -> impl Stream<Item = String> + '_ {
        self.manga_utils
            .find_all_downloades_by_manga_id(self.manga_id.clone())
    }
    pub async fn find_and_delete_all_downloades(&self) -> ManagerCoreResult<serde_json::Value> {
        self.manga_utils
            .find_and_delete_all_downloades_by_manga_id(self.manga_id.clone())
            .await
    }
    pub fn get_downloaded_cover<'a, H>(
        &'a self,
        history: &'a mut H,
    ) -> ManagerCoreResult<impl Stream<Item = String> + 'a>
    where
        H: AccessHistory,
    {
        self.manga_utils
            .get_downloaded_cover_of_a_manga(self.manga_id.clone(), history)
    }
    pub async fn get_downloaded_cover_of_a_manga_collection<'a, H>(
        &'a self,
        offset: usize,
        limit: usize,
        history: &'a mut H,
    ) -> ManagerCoreResult<Collection<String>>
    where
        H: AccessHistory,
    {
        self.manga_utils
            .get_downloaded_cover_of_a_manga_collection(
                self.manga_id.clone(),
                offset,
                limit,
                history,
            )
            .await
    }
    pub fn is_there(&self) -> ManagerCoreResult<bool> {
        self.manga_utils.is_manga_there(self.manga_id.clone())
    }
    pub fn get_manga_data(&self) -> ManagerCoreResult<ApiObject<MangaAttributes>> {
        self.manga_utils.get_manga_data_by_id(self.manga_id.clone())
    }
    pub async fn get_all_downloaded_chapter_data(
        &self,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + '_> {
        self.manga_utils
            .get_all_downloaded_chapter_data(self.manga_id.clone())
            .await
    }
    pub async fn get_downloaded_chapter(
        &self,
        offset: usize,
        limit: usize,
    ) -> ManagerCoreResult<Collection<String>> {
        self.manga_utils
            .get_downloaded_chapter_of_a_manga(self.manga_id.clone(), offset, limit)
            .await
    }
    pub fn is_cover_there(&self) -> Result<bool, std::io::Error> {
        self.manga_utils.is_manga_cover_there(self.manga_id.clone())
    }
    pub async fn aggregate_manga_chapters(&self) -> ManagerCoreResult<MangaAggregate> {
        let data = Box::pin(self.get_all_downloaded_chapter_data().await?);
        let chapters: Vec<ApiObject<ChapterAttributes>> = data.collect().await;
        let volumes = group_chapter_to_volume_aggregate(chapters)?;
        Ok(MangaAggregate {
            result: ResultType::Ok,
            volumes,
        })
    }
}

impl<'a> From<&'a MangaDownload> for MangaUtils {
    fn from(value: &'a MangaDownload) -> Self {
        Self {
            dirs_options: value.dirs_options.clone(),
            http_client_ref: value.http_client.clone(),
        }
    }
}

impl From<MangaDownload> for MangaUtils {
    fn from(value: MangaDownload) -> Self {
        Self {
            dirs_options: value.dirs_options,
            http_client_ref: value.http_client,
        }
    }
}

impl<'a> From<&'a MangaDownload> for MangaUtilsWithMangaId {
    fn from(value: &'a MangaDownload) -> Self {
        Self {
            manga_utils: From::from(value),
            manga_id: value.manga_id.to_string(),
        }
    }
}

impl From<MangaDownload> for MangaUtilsWithMangaId {
    fn from(value: MangaDownload) -> Self {
        let manga_id = value.manga_id.to_string();
        Self {
            manga_utils: From::from(value),
            manga_id,
        }
    }
}
