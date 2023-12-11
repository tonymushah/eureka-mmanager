use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};

use bytes::Bytes;
use mangadex_api_schema_rust::{v5::CoverAttributes, ApiData, ApiObject};
use uuid::Uuid;

use crate::{download::cover::CoverDownload, ManagerCoreResult};

use super::CoverUtils;

#[derive(Clone)]
pub struct CoverUtilsWithId {
    pub cover_utils: CoverUtils,
    pub(crate) cover_id: Uuid,
}

impl CoverUtilsWithId {
    pub fn new(cover_id: Uuid, cover_utils: CoverUtils) -> Self {
        Self {
            cover_utils,
            cover_id,
        }
    }
    pub fn is_there(&self) -> bool {
        self.get_data().is_ok()
    }
    pub fn is_image_there(&self) -> bool {
        self.get_image_buf_reader().is_ok()
    }
    pub fn get_data(&self) -> ManagerCoreResult<ApiObject<CoverAttributes>> {
        let data: ApiData<ApiObject<CoverAttributes>> =
            serde_json::from_reader(BufReader::new(File::open(self)?))?;
        Ok(data.data)
    }
    pub fn get_image_path(&self) -> ManagerCoreResult<PathBuf> {
        let cover_data = self.get_data()?;
        let cover_file_name = cover_data.attributes.file_name;
        let cover_file_name_path = self
            .cover_utils
            .dirs_options
            .covers_add(format!("images/{}", cover_file_name).as_str());
        Ok(Path::new(&cover_file_name_path).to_path_buf())
    }
    pub fn get_image_buf_reader(&self) -> ManagerCoreResult<BufReader<File>> {
        Ok(BufReader::new(File::open(self.get_image_path()?)?))
    }
    pub fn get_image_buf_writer(&self) -> ManagerCoreResult<BufWriter<File>> {
        Ok(BufWriter::new(File::create(self.get_image_path()?)?))
    }
    pub fn get_image_buf(&self) -> ManagerCoreResult<Bytes> {
        let mut bytes = Bytes::new();
        self.get_image_buf_reader()?.read(&mut bytes)?;
        Ok(bytes)
    }
    pub fn delete_image(&self) -> ManagerCoreResult<()> {
        std::fs::remove_file(self.get_image_path()?)?;
        Ok(())
    }
    pub fn delete(&self) -> ManagerCoreResult<()> {
        self.delete_image()?;
        std::fs::remove_file(self)?;
        Ok(())
    }
}

impl AsRef<Path> for CoverUtilsWithId {
    fn as_ref(&self) -> &Path {
        &Path::new(
            &self
                .cover_utils
                .dirs_options
                .covers_add(format!("{}.json", self.cover_id).as_str()),
        )
    }
}

impl From<CoverDownload> for CoverUtilsWithId {
    fn from(value: CoverDownload) -> Self {
        Self {
            cover_utils: CoverUtils::new(value.dirs_options, value.http_client),
            cover_id: value.cover_id,
        }
    }
}

impl<'a> From<&'a CoverDownload> for CoverUtilsWithId {
    fn from(value: &'a CoverDownload) -> Self {
        Self {
            cover_utils: CoverUtils::new(value.dirs_options.clone(), value.http_client.clone()),
            cover_id: value.cover_id,
        }
    }
}
