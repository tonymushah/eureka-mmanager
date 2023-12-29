use std::{
    fs::File,
    io::{BufReader, BufWriter, Read},
    path::{Path, PathBuf},
};

use bytes::{Bytes, BytesMut};
use mangadex_api_schema_rust::{
    v5::{CoverData, CoverObject, RelatedAttributes},
    ApiData,
};
use mangadex_api_types_rust::{RelationshipType, ResponseType, ResultType};
use uuid::Uuid;

use crate::{download::cover::CoverDownload, utils::ExtractData, ManagerCoreResult};

use super::CoverUtils;

#[derive(Clone)]
pub struct CoverUtilsWithId {
    pub cover_utils: CoverUtils,
    pub(crate) cover_id: Uuid,
}

impl ExtractData for CoverUtilsWithId {
    type Input = CoverObject;
    type Output = CoverObject;

    fn get_file_path(&self) -> ManagerCoreResult<PathBuf> {
        Ok(self.into())
    }

    fn get_data(&self) -> ManagerCoreResult<Self::Output> {
        let data: CoverData = serde_json::from_reader(self.get_buf_reader()?)?;
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
                    RelationshipType::Manga => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::Manga(_))),
                    RelationshipType::User => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::User(_))),
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
        self.delete_image()?;
        std::fs::remove_file(self.get_file_path()?)?;
        Ok(())
    }
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
        let mut bytes = BytesMut::new();
        self.get_image_buf_reader()?.read_exact(&mut bytes)?;
        Ok(bytes.into())
    }
    pub fn delete_image(&self) -> ManagerCoreResult<()> {
        std::fs::remove_file(self.get_image_path()?)?;
        Ok(())
    }
}

impl From<CoverUtilsWithId> for PathBuf {
    fn from(value: CoverUtilsWithId) -> Self {
        Path::new(
            &value
                .cover_utils
                .dirs_options
                .covers_add(format!("{}.json", value.cover_id).as_str()),
        )
        .to_path_buf()
    }
}

impl From<&CoverUtilsWithId> for PathBuf {
    fn from(value: &CoverUtilsWithId) -> Self {
        Path::new(
            &value
                .cover_utils
                .dirs_options
                .covers_add(format!("{}.json", value.cover_id).as_str()),
        )
        .to_path_buf()
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
