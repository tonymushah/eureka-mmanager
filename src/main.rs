
/*
    [x] finish the server options api
    [x] implements those modifiction to the entire app
    [x] the app can edit his settings
*/

//use std::fs;

//use mangadex_api_schema::{ApiObject, v5::ChapterAttributes};
//use mangadex_api_types::{ReferenceExpansionResource, RelationshipType};
use mangadex_desktop_api2::{launch_server_default, verify_all_fs};
//use mangadex_api::MangaDexClient;

fn main() -> anyhow::Result<()> {
    fern::Dispatch::new()
        .level(log::LevelFilter::Info)
        .chain(std::io::stdout())
        .apply().unwrap();
    verify_all_fs()?;
    launch_server_default()?;
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
