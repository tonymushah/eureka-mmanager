use std::path::{Path, PathBuf};

use async_stream::stream;
use mangadex_api_schema_rust::{
    v5::{
        ChapterAttributes, ChapterCollection, CoverAttributes, MangaAggregate, MangaData,
        MangaObject, RelatedAttributes,
    },
    ApiData, ApiObject,
};
use mangadex_api_types_rust::{RelationshipType, ResponseType, ResultType};
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::{
    download::manga::MangaDownload,
    server::traits::AccessHistory,
    utils::{
        chapter::{filter::translated_languages::filter_fn_via_translated_languages, ChapterUtils},
        collection::Collection,
        cover::CoverUtils,
        manga_aggregate::{self, MangaAggregateParams},
        ExtractData,
    },
    ManagerCoreResult,
};

use super::MangaUtils;

#[derive(Clone)]
pub struct MangaUtilsWithMangaId {
    pub(crate) manga_utils: MangaUtils,
    pub(crate) manga_id: Uuid,
}

impl ExtractData for MangaUtilsWithMangaId {
    type Input = MangaObject;
    type Output = MangaObject;

    fn get_file_path(&self) -> ManagerCoreResult<PathBuf> {
        Ok(Into::<PathBuf>::into(self))
    }

    fn get_data(&self) -> ManagerCoreResult<Self::Output> {
        let data: MangaData = serde_json::from_reader(self.get_buf_reader()?)?;
        Ok(data.data)
    }

    fn update(&self, mut input: Self::Input) -> ManagerCoreResult<()> {
        let current_data = self.get_data()?;
        let buf_writer = self.get_buf_writer()?;
        let to_input_data = {
            if input.relationships.is_empty() {
                input.relationships = current_data.relationships;
            } else {
                let contains_rels = input.relationships.iter().all(|i| match i.type_ {
                    RelationshipType::Manga => {
                        i.related.is_some()
                            && i.attributes
                                .as_ref()
                                .is_some_and(|attr| matches!(attr, RelatedAttributes::Manga(_)))
                    }
                    RelationshipType::User => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::User(_))),
                    RelationshipType::Artist => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::Author(_))),
                    RelationshipType::Author => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::Author(_))),
                    RelationshipType::Creator => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::User(_))),
                    RelationshipType::CoverArt => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::CoverArt(_))),
                    _ => false,
                });
                if !contains_rels {
                    input.relationships = current_data.relationships;
                }
            }
            ApiData {
                response: ResponseType::Entity,
                result: ResultType::Ok,
                data: input,
            }
        };
        serde_json::to_writer(buf_writer, &to_input_data)?;
        Ok(())
    }

    fn delete(&self) -> ManagerCoreResult<()> {
        tokio::runtime::Handle::current().block_on(async {
            self.delete_chapters().collect::<Vec<Uuid>>().await;
            self.delete_covers().collect::<Vec<Uuid>>().await;
        });
        std::fs::remove_file(self.get_file_path()?)?;
        Ok(())
    }
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
    pub fn find_all_downloades<'a, H: AccessHistory>(
        &'a self,
        history: &'a H,
    ) -> ManagerCoreResult<impl Stream<Item = Uuid> + 'a> {
        let stream = Box::pin(self.get_all_downloaded_chapter_data(history, Default::default()));
        Ok(stream.map(|chapter| chapter.id))
    }

    pub fn get_downloaded_covers(&self) -> impl Stream<Item = ApiObject<CoverAttributes>> + '_ {
        stream! {
            let cover_utils: CoverUtils = From::from(self.manga_utils.clone());
            if let Ok(vecs) = cover_utils.get_all_cover_data(){
                let vecs = Box::pin(vecs);
                let mut data = vecs.filter(move |data| self.is_cover_related(data));
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
    pub fn get_all_downloaded_chapter_data_default(
        &self,
    ) -> impl Stream<Item = ApiObject<ChapterAttributes>> + '_ {
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
    pub fn get_all_downloaded_chapter_data<'a, H>(
        &'a self,
        history: &'a H,
        params: MangaAggregateParams,
    ) -> impl Stream<Item = ApiObject<ChapterAttributes>> + 'a
    where
        H: AccessHistory,
    {
        let chapter_utils: ChapterUtils = From::from(self.manga_utils.clone());
        stream! {
            if let Ok(data) = chapter_utils.get_all_chapter(Some(params.additional_params), history).await {
                let data = Box::pin(chapter_utils.get_chapters_by_stream_id(Box::pin(data)));
                let mut data = data.filter_map(|next| {
                    if self.is_chapter_data_related(&next){
                        Some(next)
                    }else{
                        None
                    }
                }).filter(|item| {
                    let translated_languages = &params.translated_language;
                    if !translated_languages.is_empty() {
                        filter_fn_via_translated_languages(item, translated_languages)
                    }else {
                        true
                    }
                }).filter(|item| {
                    let groups = &params.groups;
                    if !groups.is_empty() {
                        item.relationships.iter().any(|rel| groups.contains(&rel.id))
                    } else {
                        true
                    }
                });
                while let Some(next) = data.next().await {
                    yield next;
                }
            }
        }
    }
    pub async fn get_downloaded_chapter<'a, H>(
        &'a self,
        offset: usize,
        limit: usize,
        params: MangaAggregateParams,
        history: &'a H,
    ) -> ManagerCoreResult<ChapterCollection>
    where
        H: AccessHistory,
    {
        let all_downloaded = self.get_all_downloaded_chapter_data(history, params);
        let mut data = Box::pin(all_downloaded);
        let to_use: ChapterCollection = Collection::from_async_stream(&mut data, limit, offset)
            .await?
            .try_into()?;
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
    pub async fn aggregate_manga_chapter<'a, H: AccessHistory>(
        &'a self,
        params: MangaAggregateParams,
        history: &'a H,
    ) -> ManagerCoreResult<MangaAggregate> {
        let data = Box::pin(self.get_all_downloaded_chapter_data(history, params));
        let volumes = manga_aggregate::stream::group_chapter_to_volume_aggregate(data).await;
        Ok(MangaAggregate {
            result: ResultType::Ok,
            volumes,
        })
    }
    pub async fn aggregate_manga_chapter_default(&self) -> ManagerCoreResult<MangaAggregate> {
        let data = Box::pin(self.get_all_downloaded_chapter_data_default());
        let volumes = manga_aggregate::stream::group_chapter_to_volume_aggregate(data).await;
        Ok(MangaAggregate {
            result: ResultType::Ok,
            volumes,
        })
    }
    pub fn delete_chapters(&self) -> impl Stream<Item = Uuid> + '_ {
        let stream = Box::pin(self.get_all_downloaded_chapter_data_default());
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
    pub fn delete_covers(&self) -> impl Stream<Item = Uuid> + '_ {
        let cover_utils: CoverUtils = From::from(self.manga_utils.clone());
        self.get_downloaded_covers().filter_map(move |cover| {
            if cover_utils.with_id(cover.id).delete().is_ok() {
                Some(cover.id)
            } else {
                None
            }
        })
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

impl From<MangaUtilsWithMangaId> for PathBuf {
    fn from(value: MangaUtilsWithMangaId) -> Self {
        Path::new(
            &value
                .manga_utils
                .dirs_options
                .mangas_add(format!("{}.json", value.manga_id).as_str()),
        )
        .to_path_buf()
    }
}

impl From<&MangaUtilsWithMangaId> for PathBuf {
    fn from(value: &MangaUtilsWithMangaId) -> Self {
        Path::new(
            &value
                .manga_utils
                .dirs_options
                .mangas_add(format!("{}.json", value.manga_id).as_str()),
        )
        .to_path_buf()
    }
}
