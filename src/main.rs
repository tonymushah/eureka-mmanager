/*
    [x] finish the server options api
    [x] implements those modifiction to the entire app
    [x] the app can edit his settings
*/

//use std::fs;

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::io::Write;
use std::str::FromStr;
use std::sync::Arc;

use mangadex_api_schema::{v5::ChapterAttributes, ApiObject};
//use mangadex_api_schema::{ApiObject, v5::ChapterAttributes};
//use mangadex_api_types::{ReferenceExpansionResource, RelationshipType};
use binary_search_tree::BinarySearchTree;
use mangadex_api::MangaDexClient;
use mangadex_api_types::{RelationshipType, ReferenceExpansionResource, MangaFeedSortOrder, OrderDirection};
use mangadex_desktop_api2::{
    launch_server_default, utils::manga::get_all_downloaded_manga, verify_all_fs,
};
use serde::{Deserialize, Serialize};
use std::fs::File;
use tokio::io::AsyncWriteExt;
use tokio::sync::Mutex;
use tokio::task::JoinSet;
use uuid::Uuid;
//use mangadex_api::MangaDexClient;

#[derive(Deserialize, Serialize)]
struct ChapterFeed(ApiObject<ChapterAttributes>);

impl Eq for ChapterFeed {}

impl PartialOrd for ChapterFeed {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0
            .attributes
            .readable_at
            .as_ref()
            .partial_cmp(other.0.attributes.readable_at.as_ref())
    }
}

impl AsRef<ApiObject<ChapterAttributes>> for ChapterFeed {
    fn as_ref(&self) -> &ApiObject<ChapterAttributes> {
        &self.0
    }
}

impl PartialEq for ChapterFeed {
    fn eq(&self, other: &Self) -> bool {
        self.0.id == other.0.id
    }
}

impl Ord for ChapterFeed {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        if self.0.id.cmp(&other.0.id) == Ordering::Equal {
            return Ordering::Equal;
        }
        self.0
            .attributes
            .readable_at
            .as_ref()
            .cmp(other.0.attributes.readable_at.as_ref())
    }
}

async fn collect_feed(
    manga_id: String,
    feed_: Arc<Mutex<BinarySearchTree<ChapterFeed>>>,
    client: Arc<MangaDexClient>,
) -> anyhow::Result<Uuid> {
    let id = format!("urn:uuid:{}", manga_id);
    let id = uuid::Uuid::from_str(id.as_str())?;
    let feeds = client
        .manga()
        .feed()
        .manga_id(&id)
        .order(MangaFeedSortOrder::ReadableAt(OrderDirection::Descending))
        .build()?
        .send()
        .await??;
    for feed in feeds.data {
        feed_.lock().await.insert(ChapterFeed(feed));
    }
    Ok(id)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let manga_id = get_all_downloaded_manga()?;
    let chapter_data: Arc<Mutex<BinarySearchTree<ChapterFeed>>> =
        Arc::new(Mutex::new(BinarySearchTree::new()));
    let client = Arc::new(MangaDexClient::default());
    let mut handles: JoinSet<anyhow::Result<Uuid>> = JoinSet::new();
    for id in manga_id {
        let sr = Arc::clone(&chapter_data);
        let cli = Arc::clone(&client);
        handles.spawn(async move { collect_feed(id.clone(), sr, cli).await });
    }
    while let Some(d) = handles.join_next().await {
        match d {
            Ok(d_) => match d_ {
                Ok(id) => {
                    println!("getted feed for manga {}", id);
                }
                Err(e) => {
                    eprintln!("{}", e.to_string());
                }
            },
            Err(e) => {
                eprintln!("{}", e.to_string());
            }
        }
    }
    println!("start writing");
    tokio::spawn(async move {
        let feeds = Arc::clone(&chapter_data);
        let guard = feeds.lock().await;
        let data = guard.sorted_vec();
        let value = serde_json::json!({ "data": data });
        let mut result = File::create("./result.json").unwrap();
        result.write(value.to_string().as_bytes()).unwrap();
    }).await?;

    /*let client = MangaDexClient::default();
    let getted = client
        .chapter()
        .get()
        .chapter_id(&(uuid::uuid!("urn:uuid:502d356a-0154-4429-9060-a543d544080f")))
        .include(ReferenceExpansionResource::Manga)
        .include(ReferenceExpansionResource::User)
        .include(ReferenceExpansionResource::ScanlationGroup)
        .build()?
        .send()
        .await?;
    println!("{}", serde_json::to_string(&getted)?);*/
    anyhow::Ok(())
}
