pub mod stream_filters;
mod with_id;

use mangadex_api::HttpClientRef;
use mangadex_api_input_types::manga::list::MangaListParams;
use mangadex_api_schema_rust::v5::{ChapterAttributes, MangaAttributes, MangaCollection};
use mangadex_api_schema_rust::ApiObject;
use mangadex_api_types_rust::RelationshipType;
use std::path::Path;
use std::sync::Arc;
use tokio_stream::Stream;
use tokio_stream::StreamExt;
use uuid::Uuid;

use crate::core::ManagerCoreResult;
use crate::download::chapter::ChapterDownload;
use crate::download::manga::MangaDownload;
use crate::settings::files_dirs::DirsOptions;
use crate::utils::manga::stream_filters::filter;
use crate::utils::manga::stream_filters::includes::map_fn_via_includes;

use super::chapter::ChapterUtils;
use super::collection::Collection;
use super::cover::CoverUtils;
use super::ExtractData;

pub use with_id::MangaUtilsWithMangaId;

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
    pub fn is_chap_data_related_to_manga(
        chapter: &ApiObject<ChapterAttributes>,
        manga_id: Uuid,
    ) -> bool {
        chapter
            .relationships
            .iter()
            .any(|relas| relas.type_ == RelationshipType::Manga && relas.id == manga_id)
    }
    pub fn get_manga_data_by_ids<T>(
        &'a self,
        manga_ids: T,
    ) -> impl Stream<Item = ApiObject<MangaAttributes>> + 'a
    where
        T: Stream<Item = Uuid> + std::marker::Unpin + 'a,
    {
        manga_ids.filter_map(|id| self.with_id(id).get_data().ok())
    }
    pub fn get_manga_data_by_ids_old(
        &'a self,
        manga_ids: Vec<Uuid>,
    ) -> impl Stream<Item = ApiObject<MangaAttributes>> + 'a {
        self.get_manga_data_by_ids(tokio_stream::iter(manga_ids))
    }
    pub fn get_all_downloaded_manga(&'a self) -> ManagerCoreResult<impl Stream<Item = Uuid> + 'a> {
        let path = self.dirs_options.mangas_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = std::fs::read_dir(path.as_str())?.flatten();
            Ok(tokio_stream::iter(list_dir)
                .filter_map(|file_| {
                    file_
                        .file_name()
                        .to_str()
                        .map(|data| data.to_string().replace(".json", ""))
                })
                .filter_map(|id| Uuid::parse_str(&id).ok()))
        } else {
            Err(crate::core::Error::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't find the manga directory",
            )))
        }
    }
    pub async fn get_downloaded_manga(
        &'a self,
        params: MangaListParams,
    ) -> ManagerCoreResult<MangaCollection> {
        let vecs = Box::pin(self.get_all_downloaded_manga()?);
        let manga_data = Box::pin(self.get_manga_data_by_ids(vecs));

        let collection = Collection::from_async_stream(
            manga_data
                .filter(|item| filter(item, &params))
                .map(|o| map_fn_via_includes(o, &params.includes)),
            params.limit.unwrap_or(10) as usize,
            params.offset.unwrap_or_default() as usize,
        )
        .await?;
        Ok(collection.try_into()?)
    }
    pub fn with_id(&self, manga_id: Uuid) -> MangaUtilsWithMangaId {
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
