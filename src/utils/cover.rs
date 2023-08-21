use std::{fs::File, path::Path};

use async_stream::stream;
use futures::Stream;
use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::{v5::CoverAttributes, ApiData, ApiObject};

use crate::{settings::files_dirs::DirsOptions, download::chapter::ChapterDownload};

use super::{chapter::ChapterUtils, manga::MangaUtils};

#[derive(Clone)]
pub struct CoverUtils {
    pub(crate) dirs_options: DirsOptions,
    pub(crate) http_client_ref: HttpClientRef,
}

impl CoverUtils {
    pub fn new(dirs_options: DirsOptions, http_client_ref: HttpClientRef) -> Self {
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
        match self.is_cover_there(cover_id) {
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
    pub fn get_all_cover<'a>(&'a self) -> Result<impl Stream<Item = String> + 'a, std::io::Error> {
        let file_dirs = self.dirs_options.clone();
        let path = file_dirs.covers_add("");
        if Path::new(path.as_str()).exists() {
            let list_dir = (std::fs::read_dir(path.as_str()))?;
            Ok(stream! {
                for file_ in list_dir.flatten() {
                    if let core::result::Result::Ok(metadata) = file_.metadata() {
                        if metadata.is_file() {
                            if let Some(data) = file_.file_name().to_str() {
                                yield data.to_string().replace(".json", "");
                            }
                        }
                    }
                }
            })
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "can't find the cover directory",
            ))
        }
    }
    pub fn with_id(&self, cover_id: String) -> CoverUtilsWithId {
        CoverUtilsWithId { cover_utils: self.clone(), cover_id }
    }
}

#[derive(Clone)]
pub struct CoverUtilsWithId{
    pub cover_utils : CoverUtils,
    cover_id : String
}

impl CoverUtilsWithId {
    pub fn new(cover_id : String, cover_utils : CoverUtils) -> Self{
        Self { cover_utils, cover_id }
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
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl From<MangaUtils> for CoverUtils {
    fn from(value: MangaUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl<'a> From<&'a MangaUtils> for CoverUtils {
    fn from(value: &'a MangaUtils) -> Self {
        Self::new(value.dirs_options, value.http_client_ref)
    }
}

impl From<ChapterDownload> for CoverUtils
{
    fn from(value: ChapterDownload) -> Self {
        Self { dirs_options: value.dirs_options, http_client_ref: value.http_client }
    }
}

impl<'a> From<&'a ChapterDownload> for CoverUtils
{
    fn from(value: &'a ChapterDownload) -> Self {
        Self { dirs_options: value.dirs_options, http_client_ref: value.http_client }
    }
}