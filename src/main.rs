/*
    [x] finish the server options api
    [x] implements those modifiction to the entire app
    [x] the app can edit his settings
*/

//use std::fs;

use std::io::Write;
use std::sync::Arc;

use mangadex_api_types::Language;
//use mangadex_api_schema::{ApiObject, v5::ChapterAttributes};
//use mangadex_api_types::{ReferenceExpansionResource, RelationshipType};
use mangadex_desktop_api2::feeds::get_downloaded_manga_feed;
/*use mangadex_desktop_api2::{
    launch_server_default, verify_all_fs,
};*/
use std::fs::File;
//use mangadex_api::MangaDexClient;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("Start collecting feed");

    let feed_data = get_downloaded_manga_feed(Some(Language::English)).await?;
    let errors = feed_data.1.lock().await;

    println!("{} errors", errors.len());

    for err in errors.iter(){
        println!("{}", serde_json::to_string(&err)?)
    }

    println!("start writing result");

    tokio::spawn(async move {
        let feeds = Arc::clone(&feed_data.0);
        let guard = feeds.lock().await;
        let mut data = guard.sorted_vec();
        data.reverse();
        let value = serde_json::json!({ "data": data });
        let mut result = File::create("./result.json").unwrap();
        result.write(value.to_string().as_bytes()).unwrap();
    }).await?;
    println!("Done!");

    anyhow::Ok(())
}
