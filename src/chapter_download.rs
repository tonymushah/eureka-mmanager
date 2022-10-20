// Imports used for downloading the pages to a file.
// They are not used because we're just printing the raw bytes.
use std::fs::File;
use std::io::{Write};
use uuid::Uuid;
use mangadex_api::v5::MangaDexClient;
use std::path::Path;
use serde_json::json;

pub async fn download_chapter(chapter_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let chapter_id = Uuid::parse_str(chapter_id).expect("Not a valid id");
    let chapter_top_dir = format!("chapters/{}",chapter_id.hyphenated().to_string());
    let chapter_dir = format!("chapters/{}/data", chapter_top_dir);
    std::fs::create_dir_all(format!("{}/data", chapter_id.hyphenated().to_string()))?;
    println!("chapter dir created");
    let at_home = client
        .at_home()
        .server()
        .chapter_id(&chapter_id)
        .build()?
        .send()
        .await?;

    let http_client = reqwest::Client::new();
    // puting chapter data in a json data
    if Path::new(format!("{}/data.json", chapter_top_dir).as_str()).exists() == false{
        let get_chapter = http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated().to_string())).send().await?;
        let bytes_ = get_chapter.bytes().await?;
        let mut chapter_data = File::create(format!("{}/data.json", chapter_top_dir))?;
        chapter_data.write_all(&bytes_).unwrap();
        println!("created data.json");
    }
    let mut files_: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data;
    for filename in page_filenames {
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home
            .base_url
            .join(&format!(
                "/{quality_mode}/{chapter_hash}/{page_filename}",
                quality_mode = "data",
                chapter_hash = at_home.chapter.hash,
                page_filename = filename
            ))
            .unwrap();

        let res = http_client.get(page_url).send().await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;

        // This is where you would download the file but for this example,
        // we're just printing the raw data.
        let mut file = File::create(format!("{}/{}", chapter_dir, &filename))?;
        let _ = file.write_all(&bytes);
        println!("downloaded {} ", &filename);
        files_.push(format!("{}", &filename));
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "downloaded" : files_
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
    Ok(jsons)
}
pub async fn download_chapter_saver(chapter_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let chapter_id = Uuid::parse_str(chapter_id).expect("Not a valid id");
    let chapter_top_dir = format!("chapters/{}",chapter_id.hyphenated().to_string());
    let chapter_dir = format!("{}/data-saver", chapter_top_dir);
    std::fs::create_dir_all(format!("chapters/{}/data-saver", chapter_id.hyphenated().to_string()))?;
    println!("chapter dir created");
    let at_home = client
        .at_home()
        .server()
        .chapter_id(&chapter_id)
        .build()?
        .send()
        .await?;

    let http_client = reqwest::Client::new();
    // puting chapter data in a json data
    if Path::new(format!("{}/data.json", chapter_top_dir).as_str()).exists() == false{
        let get_chapter = http_client.get(format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", mangadex_api::constants::API_URL, chapter_id.hyphenated().to_string())).send().await?;
        let bytes_ = get_chapter.bytes().await?;
        let mut chapter_data = File::create(format!("{}/data.json", chapter_top_dir))?;
        chapter_data.write_all(&bytes_).unwrap();
        println!("created data.json");
    }
    let mut files_: Vec<String> = Vec::new();
    // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
    let page_filenames = at_home.chapter.data_saver;
    for filename in page_filenames {
        // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
        let page_url = at_home
            .base_url
            .join(&format!(
                "/{quality_mode}/{chapter_hash}/{page_filename}",
                quality_mode = "data-saver",
                chapter_hash = at_home.chapter.hash,
                page_filename = filename
            ))
            .unwrap();

        let res = http_client
            .get(page_url)
            .send()
            .await?
        ;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;

        // This is where you would download the file but for this example,
        // we're just printing the raw data.
        let mut file = File::create(format!("{}/{}", chapter_dir, &filename))?;
        let _ = file.write_all(&bytes);
        println!("downloaded {} ", &filename);
        files_.push(format!("{}", &filename));
    }
    let jsons = json!({
        "result" : "ok",
        "dir" : chapter_dir,
        "downloaded" : files_
    });
    let mut file = File::create(format!("{}/{}", chapter_dir, "data.json"))?;
    let _ = file.write_all(jsons.to_string().as_bytes());
    Ok(jsons)
}

mod path{
    use std::fs::File;
    use std::io::{Write};
    use uuid::Uuid;
    use mangadex_api::v5::MangaDexClient;

    pub async fn download_chapter(path: &str, chapter_id: &str) -> anyhow::Result<()> {
        let client = MangaDexClient::default();
        let chapter_id = Uuid::parse_str(chapter_id).unwrap();
        let chapter_dir = format!("chapters/{}/{}/data", path,chapter_id.hyphenated().to_string());
        std::fs::create_dir_all(format!("{}/data", chapter_id.hyphenated().to_string()))?;
        let at_home = client
            .at_home()
            .server()
            .chapter_id(&chapter_id)
            .build()?
            .send()
            .await?;

        let http_client = reqwest::Client::new();
        // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
        let page_filenames = at_home.chapter.data;
        for filename in page_filenames {
            // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
            let page_url = at_home
                .base_url
                .join(&format!(
                    "/{quality_mode}/{chapter_hash}/{page_filename}",
                    quality_mode = "data",
                    chapter_hash = at_home.chapter.hash,
                    page_filename = filename
                ))
                .unwrap();

            let res = http_client.get(page_url).send().await?;
            // The data should be streamed rather than downloading the data all at once.
            let bytes = res.bytes().await?;

            // This is where you would download the file but for this example,
            // we're just printing the raw data.
            let mut file = File::create(format!("{}/{}", chapter_dir, &filename))?;
            let _ = file.write_all(&bytes);
            println!("downloaded {} ", &filename);
        }
        Ok(())
    }
    pub async fn download_chapter_saver(path: &str,chapter_id: &str) -> anyhow::Result<()> {
        let client = MangaDexClient::default();
        let chapter_id = Uuid::parse_str(chapter_id).unwrap();
        let chapter_dir = format!("chapters/{}/{}/data-saver", path,chapter_id.hyphenated().to_string());
        std::fs::create_dir_all(format!("{}/data-saver", chapter_id.hyphenated().to_string()))?;
        let at_home = client
            .at_home()
            .server()
            .chapter_id(&chapter_id)
            .build()?
            .send()
            .await?;

        let http_client = reqwest::Client::new();

        // Original quality. Use `.data.attributes.data_saver` for smaller, compressed images.
        let page_filenames = at_home.chapter.data_saver;
        for filename in page_filenames {
            // If using the data-saver option, use "/data-saver/" instead of "/data/" in the URL.
            let page_url = at_home
                .base_url
                .join(&format!(
                    "/{quality_mode}/{chapter_hash}/{page_filename}",
                    quality_mode = "data-saver",
                    chapter_hash = at_home.chapter.hash,
                    page_filename = filename
                ))
                .unwrap();

            let res = http_client
                .get(page_url)
                .send()
                .await?
            ;
            // The data should be streamed rather than downloading the data all at once.
            let bytes = res.bytes().await?;

            // This is where you would download the file but for this example,
            // we're just printing the raw data.
            let mut file = File::create(format!("{}/{}", chapter_dir, &filename))?;
            let _ = file.write_all(&bytes);
            println!("downloaded {} ", &filename);
        }
        Ok(())
    }
}