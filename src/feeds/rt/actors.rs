use std::future::{poll_fn, Future};
use std::str::FromStr;
use std::sync::Arc;
use std::task::Poll;

use actix::fut::wrap_future;
use actix::prelude::*;

use binary_search_tree::BinarySearchTree;
use mangadex_api::MangaDexClient;
use mangadex_api_types::{MangaFeedSortOrder, OrderDirection};
use tokio::sync::Mutex;
use tokio::task::futures::TaskLocalFuture;
use tokio::task::{JoinHandle, JoinSet, LocalSet};

use crate::utils::feed::ChapterFeed;

use crate::feeds::MangaDownloadFeedError;

use super::messages::{FeedRtMessage, FeedRtResult};

#[non_exhaustive]
pub struct FeedCollector {
    pub errors: Arc<Mutex<Vec<MangaDownloadFeedError>>>,
    pub chapter_data: Arc<Mutex<BinarySearchTree<ChapterFeed>>>,
    pub inputs: usize,
    client: Arc<MangaDexClient>,
    handles: JoinSet<()>,
    main: Option<JoinHandle<()>>,
}

impl Default for FeedCollector {
    fn default() -> Self {
        Self {
            errors: Default::default(),
            chapter_data: Arc::new(Mutex::new(BinarySearchTree::new())),
            inputs: Default::default(),
            handles: Default::default(),
            client: Arc::new(MangaDexClient::default()),
            main: None,
        }
    }
}

impl Actor for FeedCollector {
    type Context = SyncContext<Self>;

    fn started(&mut self, _ctx: &mut Self::Context) {}
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        match &self.main {
            None => {}
            Some(d) => {
                d.abort();
            }
        }
        Running::Stop
    }
}

impl Handler<FeedRtMessage> for FeedCollector {
    type Result = ();

    fn handle(&mut self, msg: FeedRtMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.inputs += 1;
        self.handle_result(msg);
    }
}

impl FeedCollector {
    fn precollect(&mut self, manga_id: String) -> Result<uuid::Uuid, MangaDownloadFeedError> {
        let id = format!("urn:uuid:{}", manga_id);

        let id: uuid::Uuid = match uuid::Uuid::from_str(id.as_str()) {
            Ok(d) => d,
            Err(e) => {
                return Err(MangaDownloadFeedError {
                    id: id.clone(),
                    error: e.to_string(),
                })
            }
        };
        Ok(id)
    }
    async fn collect(&mut self, manga_id: String) {
        let manga_id_clone = manga_id.clone();
        match self.precollect(manga_id) {
            Ok(id) => {
                let client = self.client.clone();
                let feed_ = self.chapter_data.clone();
                let builder = client.manga();
                let feeds_build = builder
                    .feed()
                    .manga_id(&id)
                    .order(MangaFeedSortOrder::ReadableAt(OrderDirection::Descending));
                /*match translated_lang {
                    Some(d_) => {
                        feeds_build = feeds_build.add_translated_language(d_)
                    },
                    None => ()
                }*/
                match feeds_build.build() {
                    Ok(d) => match d.send().await {
                        Ok(d) => match d {
                            Ok(feeds) => {
                                for feed in feeds.data {
                                    feed_.lock().await.insert(ChapterFeed::new(feed));
                                }
                            }
                            Err(error) => {
                                self.handle_result(FeedRtMessage(Err(MangaDownloadFeedError {
                                    id: manga_id_clone.clone(),
                                    error: error.to_string(),
                                })));
                            }
                        },
                        Err(error) => {
                            self.handle_result(FeedRtMessage(Err(MangaDownloadFeedError {
                                id: manga_id_clone.clone(),
                                error: error.to_string(),
                            })));
                        }
                    },
                    Err(error) => {
                        self.handle_result(FeedRtMessage(Err(MangaDownloadFeedError {
                            id: (manga_id_clone).clone(),
                            error: error.to_string(),
                        })));
                    }
                }
            }
            Err(error) => {
                self.handle_result(FeedRtMessage(Err(error)));
            }
        };
    }
    fn handle_result(&mut self, msg: FeedRtMessage) {
        let chapter_data = self.get_chapter_data();
        let error = self.get_error_arc();
        self.handles.spawn(async move {
            match msg.0 {
                Ok(d) => {
                    chapter_data.lock().await.insert(d.clone());
                }
                Err(e) => {
                    error.lock().await.push(e.clone());
                }
            }
        });
    }
    fn get_error_arc(&mut self) -> Arc<Mutex<Vec<MangaDownloadFeedError>>> {
        Arc::clone(&self.errors)
    }
    fn get_chapter_data(&mut self) -> Arc<Mutex<BinarySearchTree<ChapterFeed>>> {
        Arc::clone(&self.chapter_data)
    }
}
