use std::{str::FromStr, sync::Arc};

use binary_search_tree::BinarySearchTree;
use derive_builder::Builder;
use mangadex_api::MangaDexClient;
use mangadex_api_types_rust::{MangaFeedSortOrder, OrderDirection, Language};
use serde::{Deserialize, Serialize};
use tokio::{
    sync::Mutex,
    task::{LocalSet},
};
use uuid::Uuid;

#[cfg(feature = "feeds_rt")]
pub mod rt;

use crate::utils::{feed::ChapterFeed, manga::get_all_downloaded_manga};

async fn collect_feed(
    manga_id: String,
    feed_: Arc<Mutex<BinarySearchTree<ChapterFeed>>>,
    client: Arc<MangaDexClient>,
    translated_lang : Option<Language>
) -> anyhow::Result<Uuid> {
    let id = format!("urn:uuid:{}", manga_id);
    let id = uuid::Uuid::from_str(id.as_str())?;
    let builder = client
        .manga();
    let mut feeds_build = builder
        .feed()
        .manga_id(&id)
        .order(MangaFeedSortOrder::ReadableAt(OrderDirection::Descending));
    match translated_lang {
        Some(d_) => {
            feeds_build = feeds_build.add_translated_language(d_)
        },
        None => ()
    }
    let feeds = feeds_build
        .build()?
        .send()
        .await??;
    for feed in feeds.data {
        feed_.lock().await.insert(ChapterFeed::new(feed));
    }
    Ok(id)
}

#[derive(Builder, Serialize, Deserialize, Clone)]
pub struct MangaDownloadFeedError {
    id: String,
    error: String,
}

type MangaDownloadFeed = (Arc<Mutex<BinarySearchTree<ChapterFeed>>>, Arc<Mutex<Vec<MangaDownloadFeedError>>>);

pub async fn get_downloaded_manga_feed(translated_lang : Option<Language>) -> anyhow::Result<MangaDownloadFeed>
{
    let manga_id = get_all_downloaded_manga()?;
    let chapter_data: Arc<Mutex<BinarySearchTree<ChapterFeed>>> =
        Arc::new(Mutex::new(BinarySearchTree::new()));
    let client = Arc::new(MangaDexClient::default());
    let handles: LocalSet = LocalSet::new();
    let errors: Arc<Mutex<Vec<MangaDownloadFeedError>>> = Arc::new(Mutex::new(Vec::new()));
    for id in manga_id {
        let sr = Arc::clone(&chapter_data);
        let cli = Arc::clone(&client);
        let err = Arc::clone(&errors);
        handles.spawn_local(async move {
            let id_ = id.clone();
            match tokio::spawn(async move { collect_feed(id_, sr, cli, translated_lang).await }).await {
                Ok(t) => {
                    let id_ = id.clone();
                    match t {
                        Ok(_) => (),
                        Err(e) => {
                            match MangaDownloadFeedErrorBuilder::default()
                                .id(id_.clone())
                                .error(e.to_string())
                                .build()
                            {
                                Ok(d) => err.lock().await.push(d),
                                Err(e_) => {
                                    eprintln!("{}", e_.to_string())
                                }
                            }
                        }
                    }
                }
                Err(e) => {
                    let id_ = id.clone();
                    match MangaDownloadFeedErrorBuilder::default()
                        .id(id_.clone())
                        .error(e.to_string())
                        .build()
                    {
                        Ok(d) => err.lock().await.push(d),
                        Err(e_) => {
                            eprintln!("{}", e_.to_string())
                        }
                    }
                }
            }
            println!("collected feed for {}", id.clone());
        });
    }
    handles.await;
    drop(client);
    anyhow::Ok((Arc::clone(&chapter_data), Arc::clone(&errors)))
}
