use std::{fs::File, path::Path, sync::Arc};

use tokio_stream::{Stream, StreamExt};
use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::{v5::CoverAttributes, ApiData, ApiObject};

use crate::{
    download::{chapter::ChapterDownload, cover::CoverDownload},
    settings::files_dirs::DirsOptions,
};

use super::{chapter::ChapterUtils, manga::MangaUtils};

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
    pub(self) fn is_cover_there(&self, cover_id: String) -> Result<bool, std::io::Error> {
        if !cover_id.is_empty() {
            let path = self
                .dirs_options
                .covers_add(format!("{}.json", cover_id).as_str());
            if Path::new(path.as_str()).exists() {
                self.is_cover_image_there(cover_id)
            } else {
                std::io::Result::Ok(false)
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "the cover_id should'nt be empty",
            ))
        }
    }
    pub(self) fn is_cover_image_there(&self, cover_id: String) -> Result<bool, std::io::Error> {
        if !cover_id.is_empty() {
            let path = self
                .dirs_options
                .covers_add(format!("{}.json", cover_id).as_str());
            let cover_data: ApiData<ApiObject<CoverAttributes>> =
                serde_json::from_reader(File::open(path)?)?;
            let cover_file_name = cover_data.data.attributes.file_name;
            let cover_file_name_path = self
                .dirs_options
                .covers_add(format!("images/{}", cover_file_name).as_str());
            if Path::new(cover_file_name_path.as_str()).exists() {
                std::io::Result::Ok(true)
            } else {
                std::io::Result::Ok(false)
            }
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "the cover_id should'nt be empty",
            ))
        }
    }
    pub(self) fn get_cover_data(
        &self,
        cover_id: String,
    ) -> Result<ApiData<ApiObject<CoverAttributes>>, std::io::Error> {
        let cover_id_clone = cover_id.clone();
        match self.is_cover_there(cover_id.clone()) {
            core::result::Result::Ok(is_there) => {
                if is_there {
                    let path = self
                        .dirs_options
                        .covers_add(format!("{}.json", cover_id_clone).as_str());
                    let data: ApiData<ApiObject<CoverAttributes>> =
                        serde_json::from_str(std::fs::read_to_string(path)?.as_str())?;
                    core::result::Result::Ok(data)
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        format!("Cover {cover_id} not found"),
                    ))
                }
            }
            Err(error) => Err(error),
        }
    }
    pub fn get_all_cover(&self) -> Result<impl Stream<Item = String> + '_, std::io::Error> {
        let file_dirs = self.dirs_options.clone();
        let path = file_dirs.covers_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = (std::fs::read_dir(path.as_str()))?.flatten();
            Ok(tokio_stream::iter(list_dir).filter_map(move |file_| {
                if let core::result::Result::Ok(metadata) = file_.metadata() {
                    if metadata.is_file() {
                        file_.file_name().to_str().map(|data| data.to_string().replace(".json", ""))
                    } else {
                        None
                    }
                } else {
                    None
                }
            }))
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't find the cover directory",
            ))
        }
    }
    pub fn with_id(&self, cover_id: String) -> CoverUtilsWithId {
        CoverUtilsWithId {
            cover_utils: self.clone(),
            cover_id,
        }
    }
}

#[derive(Clone)]
pub struct CoverUtilsWithId {
    pub cover_utils: CoverUtils,
    pub(crate) cover_id: String,
}

impl CoverUtilsWithId {
    pub fn new(cover_id: String, cover_utils: CoverUtils) -> Self {
        Self {
            cover_utils,
            cover_id,
        }
    }
    pub fn is_there(&self) -> Result<bool, std::io::Error> {
        self.cover_utils.is_cover_there(self.cover_id.clone())
    }
    pub fn is_image_there(&self) -> Result<bool, std::io::Error> {
        self.cover_utils.is_cover_image_there(self.cover_id.clone())
    }
    pub fn get_data(&self) -> Result<ApiData<ApiObject<CoverAttributes>>, std::io::Error> {
        self.cover_utils.get_cover_data(self.cover_id.clone())
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

impl From<CoverDownload> for CoverUtilsWithId {
    fn from(value: CoverDownload) -> Self {
        Self {
            cover_utils: CoverUtils::new(value.dirs_options, value.http_client),
            cover_id: value.cover_id.to_string(),
        }
    }
}

impl<'a> From<&'a CoverDownload> for CoverUtilsWithId {
    fn from(value: &'a CoverDownload) -> Self {
        Self {
            cover_utils: CoverUtils::new(value.dirs_options.clone(), value.http_client.clone()),
            cover_id: value.cover_id.to_string(),
        }
    }
}
