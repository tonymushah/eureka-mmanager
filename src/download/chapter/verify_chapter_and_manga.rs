use crate::{
    server::traits::{AccessDownloadTasks, AccessHistory},
    utils::chapter::ChapterUtils,
    ManagerCoreResult,
};

use super::ChapterDownload;

impl ChapterDownload {
    pub(crate) async fn verify_chapter_and_manga<'a, T>(
        &'a self,
        ctx: &'a mut T,
    ) -> ManagerCoreResult<()>
    where
        T: AccessHistory + AccessDownloadTasks,
    {
        let chapter_utils = <ChapterUtils as From<&'a Self>>::from(self).with_id(self.chapter_id);
        self.download_json_data(ctx).await?;
        if let Ok(data) = chapter_utils.is_manga_there() {
            if !data {
                (chapter_utils).patch_manga(ctx).await?;
            }
        } else {
            (chapter_utils).patch_manga(ctx).await?;
        }
        Ok(())
    }
}
