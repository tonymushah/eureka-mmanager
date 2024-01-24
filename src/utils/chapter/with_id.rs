use std::{
    fs::{File, ReadDir},
    io::{BufReader, ErrorKind, Write},
    path::{Path, PathBuf},
};

use log::info;
use mangadex_api_schema_rust::{
    v5::{ChapterAttributes, ChapterData, ChapterObject, RelatedAttributes},
    ApiData, ApiObject,
};
use mangadex_api_types_rust::{RelationshipType, ResponseType, ResultType};
use uuid::Uuid;

use crate::{
    download::{chapter::ChapterDownload, manga::MangaDownload},
    server::traits::{AccessDownloadTasks, AccessHistory},
    settings::file_history::{
        history_w_file::traits::{
            NoLFAsyncAutoCommitRollbackInsert, NoLFAsyncAutoCommitRollbackRemove,
        },
        HistoryEntry,
    },
    utils::{manga::MangaUtils, ExtractData},
    ManagerCoreResult,
};

use super::ChapterUtils;

#[derive(Clone)]
pub struct ChapterUtilsWithID {
    pub chapter_utils: ChapterUtils,
    pub(crate) chapter_id: Uuid,
}

impl ExtractData for ChapterUtilsWithID {
    type Input = ChapterObject;
    type Output = ChapterObject;
    fn delete(&self) -> ManagerCoreResult<()> {
        std::fs::remove_dir_all(std::convert::Into::<PathBuf>::into(self))?;
        Ok(())
    }

    fn get_file_path(&self) -> ManagerCoreResult<PathBuf> {
        Ok(Path::new(
            &self
                .chapter_utils
                .dirs_options
                .chapters_add(format!("{}/data.json", self.chapter_id).as_str()),
        )
        .to_path_buf())
    }
    fn get_data(&self) -> ManagerCoreResult<Self::Output> {
        let data: ChapterData = serde_json::from_reader(self.get_buf_reader()?)?;
        Ok(data.data)
    }

    fn update(&self, mut input: Self::Input) -> ManagerCoreResult<()> {
        let current_data = self.get_data()?;
        let mut buf_writer = self.get_buf_writer()?;
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
                    RelationshipType::ScanlationGroup => i
                        .attributes
                        .as_ref()
                        .is_some_and(|attr| matches!(attr, RelatedAttributes::ScanlationGroup(_))),
                    _ => false,
                });
                if !contains_rels {
                    input.relationships = current_data.relationships;
                }
            }
            ApiData {
                result: ResultType::Ok,
                response: ResponseType::Entity,
                data: input,
            }
        };
        serde_json::to_writer(&mut buf_writer, &to_input_data)?;
        let _ = buf_writer.flush();
        Ok(())
    }
}

impl ChapterUtilsWithID {
    pub fn new(chapter_id: Uuid, chapter_utils: ChapterUtils) -> Self {
        Self {
            chapter_utils,
            chapter_id,
        }
    }
    pub fn is_manga_there(&self) -> ManagerCoreResult<bool> {
        let manga_utils: MangaUtils = From::from(self.chapter_utils.clone());
        let chap_data = self.get_data()?;
        let manga_id = chap_data
            .find_first_relationships(RelationshipType::Manga)
            .ok_or(crate::core::Error::Io(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Seems like your chapter has no manga related to him",
            )))?
            .id;
        Ok(manga_utils.with_id(manga_id).is_there())
    }
    pub async fn update<'a, T>(
        &'a self,
        ctx: &'a mut T,
    ) -> ManagerCoreResult<ApiData<ApiObject<ChapterAttributes>>>
    where
        T: AccessHistory + AccessDownloadTasks,
    {
        let entry = HistoryEntry::new(self.chapter_id, RelationshipType::Chapter);
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackInsert<HistoryEntry>>::insert(ctx, entry)
            .await?;
        let data = ChapterDownload::new(
            self.chapter_id,
            self.chapter_utils.dirs_options.clone(),
            self.chapter_utils.http_client_ref.clone(),
        )
        .download_json_data(ctx)
        .await?;
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackRemove<HistoryEntry>>::remove(ctx, entry)
            .await?;
        Ok(data)
    }
    pub async fn patch_manga<'a, T>(&'a self, ctx: &'a mut T) -> ManagerCoreResult<()>
    where
        T: AccessHistory + AccessDownloadTasks,
    {
        let manga_utils: MangaUtils = From::from(self.chapter_utils.clone());
        let chapter: ApiObject<ChapterAttributes> = self.get_data()?;
        let manga = chapter
            .find_first_relationships(RelationshipType::Manga)
            .ok_or(std::io::Error::new(
                ErrorKind::Other,
                format!("can't find manga in the chapter {}", self.chapter_id).as_str(),
            ))?;
        let manga_id = manga.id;
        let type_ = manga.type_;
        let history_entry = HistoryEntry::new(manga_id, type_);
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackInsert<HistoryEntry>>::insert(
            ctx,
            history_entry,
        )
        .await?;
        MangaDownload::new(
            manga_id,
            manga_utils.dirs_options,
            manga_utils.http_client_ref,
        )
        .download_manga(ctx)
        .await?;
        // TODO put this into actix-web
        /*
        let jsons = serde_json::json!({
            "result" : "ok",
            "type" : "manga",
            "id" : manga_id.hyphenated()
        });
         */

        info!("downloaded {}.json", manga_id.hyphenated());
        <dyn AccessHistory as NoLFAsyncAutoCommitRollbackRemove<HistoryEntry>>::remove(
            ctx,
            history_entry,
        )
        .await?;
        //Ok(jsons)
        Ok(())
    }
    fn get_path_bufs(list_dir: ReadDir) -> Vec<PathBuf> {
        list_dir
            .into_iter()
            .flatten()
            .flat_map(|file| -> std::io::Result<PathBuf> {
                let filename_os = file.file_name().clone();
                let filename = Path::new(&filename_os);
                if !filename
                    .extension()
                    .and_then(|f| f.to_str())
                    .ok_or(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "can't reconize file".to_string().to_string(),
                    ))?
                    .ends_with(".json")
                {
                    Ok(filename.to_path_buf())
                } else {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::InvalidData,
                        "can't reconize file".to_string().to_string(),
                    ))
                }
            })
            .collect()
    }
    pub fn get_data_images(&self) -> ManagerCoreResult<Vec<PathBuf>> {
        let path = self
            .chapter_utils
            .dirs_options
            .chapters_add(format!("{}/data", self.chapter_id).as_str());
        let list_dir = std::fs::read_dir(path.as_str())?;
        Ok(Self::get_path_bufs(list_dir))
    }
    pub fn get_data_saver_images(&self) -> ManagerCoreResult<Vec<PathBuf>> {
        let path = self
            .chapter_utils
            .dirs_options
            .chapters_add(format!("{}/data-saver", self.chapter_id).as_str());
        let list_dir = std::fs::read_dir(path.as_str())?;
        Ok(Self::get_path_bufs(list_dir))
    }
    pub fn get_data_image<I: AsRef<str>>(&self, image: I) -> ManagerCoreResult<BufReader<File>> {
        let path = self
            .chapter_utils
            .dirs_options
            .chapters_add(format!("{}/data/{}", self.chapter_id, image.as_ref()).as_str());
        Ok(BufReader::new(File::open(path)?))
    }
    pub fn get_data_saver_image<I: AsRef<str>>(
        &self,
        image: I,
    ) -> ManagerCoreResult<BufReader<File>> {
        let path = self
            .chapter_utils
            .dirs_options
            .chapters_add(format!("{}/data-saver/{}", self.chapter_id, image.as_ref()).as_str());
        Ok(BufReader::new(File::open(path)?))
    }
}

impl From<ChapterUtilsWithID> for PathBuf {
    fn from(value: ChapterUtilsWithID) -> Self {
        Path::new(
            &value
                .chapter_utils
                .dirs_options
                .chapters_add(value.chapter_id.to_string().as_str()),
        )
        .to_path_buf()
    }
}

impl From<&ChapterUtilsWithID> for PathBuf {
    fn from(value: &ChapterUtilsWithID) -> Self {
        Path::new(
            &value
                .chapter_utils
                .dirs_options
                .chapters_add(value.chapter_id.to_string().as_str()),
        )
        .to_path_buf()
    }
}

#[async_trait::async_trait]
pub trait AccessChapterUtisWithID:
    AccessDownloadTasks + AccessHistory + Sized + Send + Sync
{
    async fn update<'a>(
        &'a mut self,
        chapter_util_with_id: &'a ChapterUtilsWithID,
    ) -> ManagerCoreResult<ApiData<ApiObject<ChapterAttributes>>> {
        chapter_util_with_id.update(self).await
    }
    async fn patch_manga<'a>(
        &'a mut self,
        chapter_util_with_id: &'a ChapterUtilsWithID,
    ) -> ManagerCoreResult<()> {
        chapter_util_with_id.patch_manga(self).await
    }
}

impl From<ChapterDownload> for ChapterUtilsWithID {
    fn from(value: ChapterDownload) -> Self {
        let chapter_id = value.chapter_id;
        Self {
            chapter_utils: From::from(value),
            chapter_id,
        }
    }
}

impl From<&ChapterDownload> for ChapterUtilsWithID {
    fn from(value: &ChapterDownload) -> Self {
        Self {
            chapter_utils: From::from(value),
            chapter_id: value.chapter_id,
        }
    }
}
