use std::io::Write;

use mangadex_api::MangaDexClient;
use mangadex_api_schema_rust::{v5::ChapterAttributes, ApiData, ApiObject};
use mangadex_api_types_rust::ReferenceExpansionResource;

use crate::{
    server::traits::AccessDownloadTasks,
    utils::{chapter::ChapterUtilsWithID, ExtractData},
    ManagerCoreResult,
};

use super::ChapterDownload;

impl ChapterDownload {
    pub async fn download_json_data<'a, D>(
        &'a self,
        task_manager: &'a mut D,
    ) -> ManagerCoreResult<ApiData<ApiObject<ChapterAttributes>>>
    where
        D: AccessDownloadTasks,
    {
        let chapter_utils_with_id: ChapterUtilsWithID = self.into();
        let client = MangaDexClient::new_with_http_client_ref(self.http_client.clone());
        let id = self.chapter_id;
        task_manager
            .lock_spawn_with_data(async move {
                let get_chapter = client
                    .chapter()
                    .id(id)
                    .get()
                    .include(ReferenceExpansionResource::Manga)
                    .include(ReferenceExpansionResource::ScanlationGroup)
                    .include(ReferenceExpansionResource::User)
                    .send()
                    .await?;

                let mut writer = chapter_utils_with_id.get_buf_writer()?;
                serde_json::to_writer(&mut writer, &get_chapter)?;
                writer.flush()?;
                Ok(get_chapter)
            })
            .await?
    }
}
