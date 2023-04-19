use actix::prelude::*;

use binary_search_tree::BinarySearchTree;
use std::sync::Mutex;
use tokio::task::{JoinSet, JoinHandle};

use crate::utils::feed::ChapterFeed;

use crate::feeds::MangaDownloadFeedError;

use super::messages::{FeedRtMessage, FeedRtResult};

#[non_exhaustive]
pub struct FeedRtActor {
    pub errors: Mutex<Vec<MangaDownloadFeedError>>,
    pub chapter_data: Mutex<BinarySearchTree<ChapterFeed>>,
    pub inputs: usize,
    handles: Mutex<JoinSet<()>>,
    main : Option<JoinHandle<()>>
}

impl Default for FeedRtActor {
    fn default() -> Self {
        Self {
            errors: Default::default(),
            chapter_data: Mutex::new(BinarySearchTree::new()),
            inputs: Default::default(),
            handles: Default::default(),
            main: None
        }
    }
}

impl Actor for FeedRtActor {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {

    }
    fn stopping(&mut self, _ctx: &mut Self::Context) -> Running {
        match &self.main {
            None => {},
            Some(d) => {
                d.abort();
            }
        }
        Running::Stop
    }
}

impl Handler<FeedRtMessage> for FeedRtActor {
    type Result = FeedRtResult;

    fn handle(&mut self, msg: FeedRtMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.inputs += 1;
        let dd = msg.0;
        match dd {
            Ok(d) => {
                match self.chapter_data.get_mut() {
                    Ok(_d) => {
                        _d.insert(d.clone());
                    }
                    Err(_) => (),
                };
                Ok(d.clone())
            }
            Err(e) => {
                match self.errors.get_mut() {
                    Ok(_d) => {
                        _d.push(e.clone());
                    }
                    Err(_) => (),
                };
                Err(e.clone())
            }
        }
    }
}
