use uuid::Uuid;

use crate::ManagerCoreResult;

use super::MangaUtils;

#[derive(Clone)]
pub struct MangaUtilsWithMangaId {
    pub(crate) manga_utils: MangaUtils,
    pub(crate) manga_id: Uuid,
}

impl MangaUtilsWithMangaId {
    pub fn new(id: Uuid, utils: MangaUtils) -> Self {
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
    pub fn is_chapter_data_related(&self, chapter: ApiObject<ChapterAttributes>) -> bool {
        MangaUtils::is_chap_data_related_to_manga(&chapter, self.manga_id.clone())
    }
    pub fn find_all_downloades(&self) -> ManagerCoreResult<impl Stream<Item = String> + '_> {
        let stream = Box::pin(
            self.manga_utils
                .get_all_downloaded_chapter_data(self.manga_id.clone())?,
        );
        Ok(stream.map(|chapter| chapter.id.to_string()))
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
    pub fn get_all_downloaded_chapter_data(
        &self,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + '_> {
        self.manga_utils
            .get_all_downloaded_chapter_data(self.manga_id.clone())
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
        self.aggregate_manga_chapters_async_friendly().await
    }
    pub async fn aggregate_manga_chapters_async_friendly(
        &self,
    ) -> ManagerCoreResult<MangaAggregate> {
        let data = Box::pin(self.get_all_downloaded_chapter_data()?);
        let volumes = super::manga_aggregate::stream::group_chapter_to_volume_aggregate(data).await;
        Ok(MangaAggregate {
            result: ResultType::Ok,
            volumes,
        })
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
