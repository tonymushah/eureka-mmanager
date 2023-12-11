use std::{
    fs::File,
    io::BufReader,
    path::Path,
};

use async_stream::stream;
use mangadex_api_schema_rust::{
    v5::{ChapterAttributes, CoverAttributes, MangaAggregate, MangaAttributes},
    ApiData, ApiObject,
};
use mangadex_api_types_rust::{RelationshipType, ResultType};
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::{
    download::manga::MangaDownload,
    server::traits::AccessHistory,
    utils::{chapter::ChapterUtils, collection::Collection, cover::CoverUtils, manga_aggregate},
    ManagerCoreResult,
};

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
    pub async fn is_chap_related(&self, chap_id: Uuid) -> ManagerCoreResult<bool> {
        let chapter_utils: ChapterUtils = From::from(self.manga_utils.clone());
        let chapter: ApiObject<ChapterAttributes> = chapter_utils.with_id(chap_id).get_data()?;

        Ok(self.is_chapter_data_related(&chapter))
    }
    pub fn is_chapter_data_related(&self, chapter: &ApiObject<ChapterAttributes>) -> bool {
        MangaUtils::is_chap_data_related_to_manga(chapter, self.manga_id)
    }
    pub fn find_all_downloades(&self) -> ManagerCoreResult<impl Stream<Item = Uuid> + '_> {
        let stream = Box::pin(self.get_all_downloaded_chapter_data());
        Ok(stream.map(|chapter| chapter.id))
    }

    pub fn get_downloaded_covers<'a>(
        &'a self,
    ) -> impl Stream<Item = ApiObject<CoverAttributes>> + 'a {
        stream! {
            let cover_utils: CoverUtils = From::from(self.manga_utils.clone());
            if let Ok(vecs) = cover_utils.get_all_cover_data(){
                let vecs = Box::pin(vecs);
                let mut data = vecs.filter(move |data| self.is_cover_related(&data));
                while let Some(data) = data.next().await{
                    yield data;
                }
            };
        }
    }
    pub async fn get_downloaded_cover_of_a_manga_collection(
        &self,
        offset: usize,
        limit: usize,
    ) -> ManagerCoreResult<Collection<ApiObject<CoverAttributes>>> {
        Collection::from_async_stream(self.get_downloaded_covers(), limit, offset).await
    }
    pub fn is_there(&self) -> bool {
        self.get_data().is_ok()
    }
    pub fn get_data(&self) -> ManagerCoreResult<ApiObject<MangaAttributes>> {
        let data: ApiData<ApiObject<MangaAttributes>> =
            serde_json::from_reader(BufReader::new(File::open(self)?))?;
        Ok(data.data)
    }
    pub fn get_all_downloaded_chapter_data<'a>(
        &'a self,
    ) -> impl Stream<Item = ApiObject<ChapterAttributes>> + 'a {
        let chapter_utils: ChapterUtils = From::from(self.manga_utils.clone());
        stream! {
            if let Ok(data) = chapter_utils.get_all_chapters_data() {
                let data = Box::pin(data);
                let mut data = data.filter_map(|next| {
                    if self.is_chapter_data_related(&next){
                        Some(next)
                    }else{
                        None
                    }
                });
                while let Some(next) = data.next().await {
                    yield next;
                }
            }
        }
    }
    pub async fn get_downloaded_chapter(
        &self,
        offset: usize,
        limit: usize,
    ) -> ManagerCoreResult<Collection<Uuid>> {
        let all_downloaded = self.get_all_downloaded_chapter_data();
        let mut data = Box::pin(all_downloaded);
        let to_use: Collection<Uuid> = Collection::from_async_stream(&mut data, limit, offset)
            .await?
            .convert_to(|d| d.id)?;
        Ok(to_use)
    }
    pub fn is_cover_there(&self) -> ManagerCoreResult<bool> {
        let manga_data = self.get_data()?;
        let cover_id = manga_data
            .find_first_relationships(RelationshipType::CoverArt)
            .ok_or(std::io::Error::new(
                std::io::ErrorKind::Other,
                "this manga has no cover_art",
            ))?
            .id;
        let cover_utils: CoverUtils = From::from(self.manga_utils.clone());
        Ok(cover_utils.with_id(cover_id).is_there())
    }
    pub fn is_cover_id_related(&self, cover_id: Uuid) -> ManagerCoreResult<bool> {
        let cover_utils: CoverUtils = From::from(self.manga_utils.clone());
        Ok(self.is_cover_related(&cover_utils.with_id(cover_id).get_data()?))
    }
    pub fn is_cover_related(&self, cover: &ApiObject<CoverAttributes>) -> bool {
        cover
            .relationships
            .iter()
            .any(|rel| rel.type_ == RelationshipType::Manga && rel.id == self.manga_id)
    }
    pub async fn aggregate_manga_chapters(&self) -> ManagerCoreResult<MangaAggregate> {
        self.aggregate_manga_chapters_async_friendly().await
    }
    pub async fn aggregate_manga_chapters_async_friendly(
        &self,
    ) -> ManagerCoreResult<MangaAggregate> {
        let data = Box::pin(self.get_all_downloaded_chapter_data());
        let volumes = manga_aggregate::stream::group_chapter_to_volume_aggregate(data).await;
        Ok(MangaAggregate {
            result: ResultType::Ok,
            volumes,
        })
    }
    pub fn delete_chapters<'a>(&'a self) -> impl Stream<Item = Uuid> + 'a {
        let stream = Box::pin(self.get_all_downloaded_chapter_data());
        stream.filter_map(|chapter| {
            if std::fs::remove_dir_all(
                self.manga_utils
                    .dirs_options
                    .chapters_add(chapter.id.to_string().as_str()),
            )
            .is_ok()
            {
                Some(chapter.id)
            } else {
                None
            }
        })
    }
    pub fn delete_covers<'a>(&'a self) -> impl Stream<Item = Uuid> + 'a {
        let cover_utils: CoverUtils = From::from(self.manga_utils.clone());
        self.get_downloaded_covers().filter_map(move |cover| {
            if cover_utils.with_id(cover.id).delete().is_ok() {
                Some(cover.id)
            } else {
                None
            }
        })
    }
    pub async fn delete(&self) -> ManagerCoreResult<()> {
        self.delete_chapters().collect::<Vec<Uuid>>().await;
        self.delete_covers().collect::<Vec<Uuid>>().await;
        std::fs::remove_file(self)?;
        Ok(())
    }
}

impl<'a> From<&'a MangaDownload> for MangaUtilsWithMangaId {
    fn from(value: &'a MangaDownload) -> Self {
        Self {
            manga_utils: From::from(value),
            manga_id: value.manga_id,
        }
    }
}

impl From<MangaDownload> for MangaUtilsWithMangaId {
    fn from(value: MangaDownload) -> Self {
        let manga_id = value.manga_id;
        Self {
            manga_utils: From::from(value),
            manga_id,
        }
    }
}

impl AsRef<Path> for MangaUtilsWithMangaId {
    fn as_ref(&self) -> &Path {
        &Path::new(
            &self
                .manga_utils
                .dirs_options
                .mangas_add(format!("{}.json", self.manga_id).as_str()),
        )
    }
}
