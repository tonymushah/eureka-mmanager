use std::{fs::File, io::{BufReader, ErrorKind}, path::Path};

use log::info;
use mangadex_api_schema_rust::{v5::ChapterAttributes, ApiData, ApiObject};
use mangadex_api_types_rust::RelationshipType;
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
    utils::manga::MangaUtils,
    ManagerCoreResult,
};

use super::ChapterUtils;

#[derive(Clone)]
pub struct ChapterUtilsWithID {
    pub chapter_utils: ChapterUtils,
    pub(crate) chapter_id: Uuid,
}

impl ChapterUtilsWithID {
    pub fn new(chapter_id: Uuid, chapter_utils: ChapterUtils) -> Self {
        Self {
            chapter_utils,
            chapter_id,
        }
    }
    pub fn is_manga_there(&self) -> ManagerCoreResult<bool> {
        let manga_utils: MangaUtils = From::from(self.chapter_utils);
        let chap_data = self.get_data()?;
        let manga_id = chap_data
            .find_first_relationships(RelationshipType::Manga)
            .ok_or(crate::core::Error::Io(
                std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Seems like your chapter has no manga related to him",
                ),
            ))?
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
    pub async fn patch_manga<'a, T>(
        &'a self,
        ctx: &'a mut T,
    ) -> ManagerCoreResult<()>
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
    pub fn get_data(&self) -> ManagerCoreResult<ApiObject<ChapterAttributes>> {
        let path = self
            .chapter_utils
            .dirs_options
            .chapters_add(format!("{}/data.json", self.chapter_id).as_str());
        let data: ApiData<ApiObject<ChapterAttributes>> =
            serde_json::from_reader(BufReader::new(File::open(path)?))?;
        Ok(data.data)
    }
    pub fn delete(&self) -> ManagerCoreResult<()> {
        std::fs::remove_dir_all(self)?;
        Ok(())
    }
}

impl AsRef<Path> for ChapterUtilsWithID {
    fn as_ref(&self) -> &Path {
        &Path::new(&self
            .chapter_utils
            .dirs_options
            .chapters_add(self.chapter_id.to_string().as_str()))
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
