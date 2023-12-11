mod with_id;

use std::{fs::Metadata, path::Path, sync::Arc};

use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::{v5::CoverAttributes, ApiObject};
use tokio_stream::{Stream, StreamExt};
use uuid::Uuid;

use crate::{
    download::chapter::ChapterDownload, settings::files_dirs::DirsOptions, ManagerCoreResult,
};

use super::{chapter::ChapterUtils, manga::MangaUtils};

pub use with_id::CoverUtilsWithId;

#[derive(Clone)]
pub struct CoverUtils {
    pub(crate) dirs_options: Arc<DirsOptions>,
    pub(crate) http_client_ref: HttpClientRef,
}

impl CoverUtils {
    pub fn new(dirs_options: Arc<DirsOptions>, http_client_ref: HttpClientRef) -> Self {
        Self {
            dirs_options,
            http_client_ref,
        }
    }
    pub fn get_all_cover(&self) -> Result<impl Stream<Item = Uuid> + '_, std::io::Error> {
        let file_dirs = self.dirs_options.clone();
        let path = file_dirs.covers_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = std::fs::read_dir(path.as_str())?.flatten();
            Ok(tokio_stream::iter(list_dir)
                .filter_map(move |file_| {
                    if file_.metadata().ok().map(|e| e.is_file()).unwrap_or(false) {
                        file_
                            .file_name()
                            .to_str()
                            .map(|data| data.replace(".json", ""))
                    } else {
                        None
                    }
                })
                .filter_map(|entry| Uuid::parse_str(&entry).ok()))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't find the cover directory",
            ))
        }
    }
    pub fn get_all_cover_data<'a>(
        &'a self,
    ) -> ManagerCoreResult<impl Stream<Item = ApiObject<CoverAttributes>> + 'a> {
        Ok(Box::pin(self.get_all_cover()?).filter_map(|id| self.with_id(id).get_data().ok()))
    }
    pub fn with_id(&self, cover_id: Uuid) -> CoverUtilsWithId {
        CoverUtilsWithId {
            cover_utils: self.clone(),
            cover_id,
        }
    }
}

impl From<ChapterUtils> for CoverUtils {
    fn from(value: ChapterUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a ChapterUtils> for CoverUtils {
    fn from(value: &'a ChapterUtils) -> Self {
        Self::new(value.dirs_options.clone(), value.http_client_ref.clone())
    }
}

impl From<MangaUtils> for CoverUtils {
    fn from(value: MangaUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a MangaUtils> for CoverUtils {
    fn from(value: &'a MangaUtils) -> Self {
        Self::new(value.dirs_options.clone(), value.http_client_ref.clone())
    }
}

impl From<ChapterDownload> for CoverUtils {
    fn from(value: ChapterDownload) -> Self {
        Self {
            dirs_options: value.dirs_options,
            http_client_ref: value.http_client,
        }
    }
}

impl<'a> From<&'a ChapterDownload> for CoverUtils {
    fn from(value: &'a ChapterDownload) -> Self {
        Self {
            dirs_options: value.dirs_options.clone(),
            http_client_ref: value.http_client.clone(),
        }
    }
}
