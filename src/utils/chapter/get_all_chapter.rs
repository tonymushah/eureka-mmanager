mod async_get_all_chapter;
mod not_include_fails;
mod only_fails;

use crate::methods::get::_find_all_downloaded_chapter::GetChapterQuery;

pub use async_get_all_chapter::AsyncGetAllChapter;
pub use not_include_fails::NotIncludeFails;
pub use only_fails::OnlyFails;
use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
#[serde(default)]
pub struct GetAllChapter {
    pub include_fails: bool,
    pub only_fails: bool,
}

impl Default for GetAllChapter {
    fn default() -> Self {
        Self {
            include_fails: true,
            only_fails: false,
        }
    }
}

impl From<GetChapterQuery> for GetAllChapter {
    fn from(value: GetChapterQuery) -> Self {
        Self {
            include_fails: value.include_fails.unwrap_or(true),
            only_fails: value.only_fails.unwrap_or(false),
        }
    }
}

#[cfg(test)]
mod test {
    use tokio_stream::StreamExt;

    use crate::{
        server::{traits::AccessHistory, AppState},
        utils::chapter::get_all_chapter::NotIncludeFails,
    };

    #[tokio::test]
    async fn test_not_fails() {
        let app_state = AppState::init().await.unwrap();
        let binding = app_state.chapter_utils();
        let all_chapter = Box::pin(binding.get_all_chapter_without_history().unwrap());
        let history = app_state
            .get_history_w_file_by_rel_or_init(mangadex_api_types_rust::RelationshipType::Chapter)
            .await
            .unwrap();
        let stream =
            NotIncludeFails::new(all_chapter, history.owned_read_history().unwrap().clone());
        let mut stream = Box::pin(stream);
        while let Some(id) = stream.next().await {
            println!("{id}")
        }
    }
}
