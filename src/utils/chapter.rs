use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::{
    v5::{ChapterAttributes, ChapterCollection, ChapterObject},
    ApiObject,
};
use mangadex_api_types_rust::RelationshipType;
use std::{path::Path, sync::Arc};
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::{
    core::{Error, ManagerCoreResult},
    download::chapter::ChapterDownload,
    methods::get::_find_all_downloaded_chapter::GetChapterQuery,
    server::traits::AccessHistory,
    settings::{file_history::HistoryWFile, files_dirs::DirsOptions},
};

use self::{
    filter::filter,
    get_all_chapter::{AsyncGetAllChapter, NotIncludeFails, OnlyFails},
};

use super::{collection::Collection, cover::CoverUtils, manga::MangaUtils, ExtractData};

pub mod filter;
mod get_all_chapter;
mod with_id;

pub use get_all_chapter::GetAllChapter;
pub use with_id::{AccessChapterUtisWithID, ChapterUtilsWithID};

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
    pub fn get_all_chapters_data(
        &self,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<ChapterAttributes>> + '_> {
        Ok(self.get_chapters_by_stream_id(Box::pin(self.get_all_chapter_without_history()?)))
    }
    pub fn get_chapters_by_vec_id(
        &self,
        chap_ids: Vec<Uuid>,
    ) -> impl Stream<Item = ApiObject<ChapterAttributes>> + '_ {
        tokio_stream::iter(chap_ids).filter_map(move |id| {
            if let Ok(data_) = self.with_id(id).get_data() {
                Some(data_)
            } else {
                None
            }
        })
    }
    pub fn get_chapters_by_stream_id<'a, S>(
        &'a self,
        stream: S,
    ) -> impl Stream<Item = ApiObject<ChapterAttributes>> + 'a
    where
        S: Stream<Item = Uuid> + Unpin + 'a,
    {
        stream.filter_map(move |id| {
            if let Ok(data_) = self.with_id(id).get_data() {
                Some(data_)
            } else {
                None
            }
        })
    }
    pub fn get_all_chapter_without_history(
        &self,
    ) -> ManagerCoreResult<impl Stream<Item = Uuid> + '_> {
        let file_dirs = self.dirs_options.clone();
        let path = file_dirs.chapters_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = std::fs::read_dir(path.as_str())?;
            Ok(tokio_stream::iter(list_dir.flatten())
                .filter_map(move |files| {
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
                })
                .filter_map(|input| {
                    if let Ok(id) = Uuid::parse_str(&input) {
                        Some(id)
                    } else {
                        None
                    }
                }))
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
    ) -> ManagerCoreResult<impl Stream<Item = Uuid> + 'a>
    where
        H: AccessHistory,
    {
        let parameters = parameters.unwrap_or_default();
        let file_dirs = self.dirs_options.clone();
        let all_chapters = Box::pin(self.get_all_chapter_without_history()?);

        let hist: HistoryWFile = history
            .get_history_w_file_by_rel_or_init(RelationshipType::Chapter)
            .await?;
        let h = hist.owned_read_history()?;
        let re_h = h.clone();
        Ok(AsyncGetAllChapter {
            only_fails: OnlyFails::new(tokio_stream::iter(re_h).filter_map(move |entry| {
                if Path::new(
                    format!(
                        "{}/data.json",
                        file_dirs.chapters_add(entry.to_string().as_str())
                    )
                    .as_str(),
                )
                .exists()
                {
                    Some(entry)
                } else {
                    None
                }
            })),
            parameters,
            not_fails: NotIncludeFails::new(all_chapters, h.clone()),
        })
    }
    pub async fn get_all_downloaded_chapters<'a, H>(
        &'a self,
        parameters: Option<GetChapterQuery>,
        history: &'a mut H,
    ) -> ManagerCoreResult<ChapterCollection>
    where
        H: AccessHistory,
    {
        if let Some(param) = parameters {
            let stream = self
                .get_all_chapter(Some(GetAllChapter::from(param.clone())), history)
                .await?;
            let stream = Box::pin(self.get_chapters_by_stream_id(Box::pin(stream)));
            let stream = stream.filter(|item| filter(item, &param.params));
            let collection: Collection<ChapterObject> = Collection::from_async_stream(
                stream,
                param.clone().params.limit.unwrap_or(10) as usize,
                param.params.offset.unwrap_or(0) as usize,
            )
            .await?;
            Ok(collection.try_into()?)
        } else {
            let stream = self.get_all_chapter(None, history).await?;
            let stream = self.get_chapters_by_stream_id(Box::pin(stream));
            let collection: Collection<ChapterObject> =
                Collection::from_async_stream(stream, 10, 0).await?;
            Ok(collection.try_into()?)
        }
    }
    pub fn with_id(&self, chapter_id: Uuid) -> ChapterUtilsWithID {
        ChapterUtilsWithID {
            chapter_utils: self.clone(),
            chapter_id,
        }
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

#[cfg(test)]
mod tests {

    use crate::utils::ExtractData;

    use super::*;
    #[tokio::test]
    pub async fn test_get_chapter_by_id() {
        let data = ChapterUtils::new(
            Arc::new(DirsOptions::new().unwrap()),
            HttpClientRef::default(),
        )
        .with_id(Uuid::parse_str("167fb8f3-1180-4b1c-ac02-a01dc24b8865").unwrap())
        .get_data()
        .unwrap();
        println!("{}", serde_json::to_string(&data).unwrap());
    }
    #[tokio::test]
    pub async fn test_get_chapters_by_vec_ids() {
        let dir_options = Arc::new(DirsOptions::new().unwrap());
        let client = HttpClientRef::default();
        let chapter_utils = ChapterUtils::new(dir_options, client);
        let manga_utils: MangaUtils = From::from(chapter_utils.clone());
        let manga_id = Uuid::parse_str("17727b0f-c9f2-4ab5-a0b1-b7b0cf6c1fc8").unwrap();
        let this_manga_utils = manga_utils.with_id(manga_id);
        let manga_downloads = Box::pin(this_manga_utils.find_all_downloades().unwrap());
        let datas = chapter_utils.get_chapters_by_stream_id(manga_downloads);
        tokio::pin!(datas);
        while let Some(chap) = datas.next().await {
            println!("{}", serde_json::to_string(&chap).unwrap());
        }
    }
}
