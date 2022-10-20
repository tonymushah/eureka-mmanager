// Imports used for downloading the cover to a file.
// They are not used because we're just printing the raw bytes.
use std::fs::File;
use std::io::Write;

use anyhow::Ok;
use reqwest::Url;
use uuid::Uuid;
use mangadex_api::types::RelationshipType;
use mangadex_api::v5::MangaDexClient;
use mangadex_api::CDN_URL;

pub async fn cover_download_by_manga_id(manga_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let manga_id = Uuid::parse_str(manga_id).unwrap();
    let manga = client
        .manga()
        .get()
        .manga_id(&manga_id)
        .build()?
        .send()
        .await?;
    let http_client = reqwest::Client::new();
    let cover_id = manga
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::CoverArt)
        .expect("no cover art found for manga")
        .id;
    let cover = client
        .cover()
        .get()
        .cover_id(&cover_id)
        .build()?
        .send()
        .await?;

    // This uses the best quality image.
    // To use smaller, thumbnail-sized images, append any of the following:
    //
    // - .512.jpg
    // - .256.jpg
    //
    // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
    let cover_url = Url::parse(&format!(
            "{}/covers/{}/{}",
            CDN_URL, manga_id, cover.data.attributes.file_name
        ))
        .unwrap();

    
    
    let res = http_client.get(cover_url).send().await?;
    // The data should be streamed rather than downloading the data all at once.
    let bytes = res.bytes().await?;
    let filename = cover.data.attributes.file_name;
    // This is where you would download the file but for this example, we're just printing the raw data.
    let mut file = File::create(format!("covers/images/{}", filename.as_str()))?;
    let _ = file.write_all(&bytes);

    let resps = http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_id.hyphenated())).send().await?;
    let mut files = File::create(format!("covers/{}.json", cover_id.hyphenated()))?;

    files.write_all(&resps.bytes().await?)?;

    println!("downloaded {}", filename.as_str());
    Ok(serde_json::json!({
        "result" : "ok",
        "type": "cover",
        "downloded" : cover_id.hyphenated()
    }))
}

pub async fn cover_download_quality_by_manga_id(manga_id: &str, quality:  u32) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let manga_id = Uuid::parse_str(manga_id).unwrap();
    let manga = client
        .manga()
        .get()
        .manga_id(&manga_id)
        .build()?
        .send()
        .await?;
    let http_client = reqwest::Client::new();

    let cover_id = manga
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::CoverArt)
        .expect("no cover art found for manga")
        .id;

    let cover = client
        .cover()
        .get()
        .cover_id(&cover_id)
        .build()?
        .send()
        .await?;

    // This uses the best quality image.
    // To use smaller, thumbnail-sized images, append any of the following:
    //
    // - .512.jpg
    // - .256.jpg
    //
    // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
    if quality == 256 || quality == 512 {
        // This uses the best quality image.
        // To use smaller, thumbnail-sized images, append any of the following:
        //
        // - .512.jpg
        // - .256.jpg
        //
        // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
        let cover_url = Url::parse(&format!(
                "{}/covers/{}/{}",
                CDN_URL, manga_id, format!("{}.{}.jpg", cover.data.attributes.file_name, quality)
            ))
            .unwrap();

        let res = http_client.get(cover_url).send().await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;
        let filename = cover.data.attributes.file_name;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let mut file = File::create(format!("covers/images/{}", filename.as_str()))?;
        let _ = file.write_all(&bytes);

        let resps = http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_id.hyphenated())).send().await?;
        let mut files = File::create(format!("covers/{}.json", cover_id.hyphenated()))?;

        files.write_all(&resps.bytes().await?)?;
        println!("downloaded {}", filename.as_str());
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id.hyphenated()
        }))
    }else {
        Err(anyhow::Error::msg("not a valid size"))
    }
    
}

pub async fn cover_download_by_cover(cover_id: &str) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let cover = client
        .cover()
        .get()
        .cover_id(&Uuid::parse_str(cover_id).unwrap())
        .build()?
        .send()
        .await?;

    let http_client = reqwest::Client::new();
    
    

    let manga_id = cover
        .data
        .relationships
        .iter()
        .find(
            | related 
            | related.type_ == RelationshipType::Manga
        )
        .expect("No manga found")
        .id;
    // This uses the best quality image.
    // To use smaller, thumbnail-sized images, append any of the following:
    //
    // - .512.jpg
    // - .256.jpg
    //
    // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
    let cover_url = Url::parse(&format!(
            "{}/covers/{}/{}",
            CDN_URL, manga_id, cover.data.attributes.file_name
        ))
        .unwrap();

    

    let res = http_client.get(cover_url).send().await?;
    // The data should be streamed rather than downloading the data all at once.
    let bytes = res.bytes().await?;
    let filename = cover.data.attributes.file_name;
    // This is where you would download the file but for this example, we're just printing the raw data.
    let mut file = File::create(format!("covers/images/{}", filename.as_str()))?;
    let _ = file.write_all(&bytes);
    println!("downloaded {}", filename.as_str());
    let resps = http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_id)).send().await?;
    let mut files = File::create(format!("covers/{}.json", cover_id))?;

    files.write_all(&resps.bytes().await?)?;
    Ok(serde_json::json!({
        "result" : "ok",
        "type": "cover",
        "downloded" : cover_id
    }))
}

pub async fn cover_download_quality_by_cover(cover_id: &str, quality:  u32) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let cover = client
        .cover()
        .get()
        .cover_id(&Uuid::parse_str(cover_id).unwrap())
        .build()?
        .send()
        .await?;
    let http_client = reqwest::Client::new();
    let manga_id = cover
        .data
        .relationships
        .iter()
        .find(
            | related 
            | related.type_ == RelationshipType::Manga
        )
        .expect("No manga found")
        .id;
    if quality == 256 || quality == 512 {
        // This uses the best quality image.
        // To use smaller, thumbnail-sized images, append any of the following:
        //
        // - .512.jpg
        // - .256.jpg
        //
        // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
        let cover_url = Url::parse(&format!(
                "{}/covers/{}/{}",
                CDN_URL, manga_id, format!("{}.{}.jpg", cover.data.attributes.file_name, quality)
            ))
            .unwrap();

        let res = http_client.get(cover_url).send().await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;
        let filename = cover.data.attributes.file_name;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let mut file = File::create(format!("covers/images/{}", filename.as_str()))?;
        let _ = file.write_all(&bytes);
        
        let resps = http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_id)).send().await?;
        let mut files = File::create(format!("covers/{}.json", cover_id))?;

        files.write_all(&resps.bytes().await?)?;

        println!("downloaded {}", filename.as_str());
        //    println!("downloaded {}", filename.as_str());
        Ok(serde_json::json!({
            "result" : "ok",
            "type": "cover",
            "downloded" : cover_id
        }))
    }else{
        Err(anyhow::Error::msg("not a valid size"))
    }
}

pub async fn all_covers_download_quality_by_manga_id(manga_id: &str, limit: u32) -> anyhow::Result<serde_json::Value> {
    let client = MangaDexClient::default();
    let manga_id = Uuid::parse_str(manga_id).expect("Not a valid id");

    let covers = client
        .cover()
        .list()
        .add_manga_id(&manga_id)
        .limit(limit)
        .build()?
        .send()
        .await?;
    let http_client = reqwest::Client::new();
    let mut vecs : Vec<String> = Vec::new();
    for cover_to_use in covers.data{
        let cover_url = Url::parse(&format!(
                "{}/covers/{}/{}",
                CDN_URL, manga_id, cover_to_use.attributes.file_name
            ))
            .unwrap();
        let res = http_client.get(cover_url).send().await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;
        let filename = cover_to_use.attributes.file_name;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let mut file = File::create(format!("covers/images/{}", filename.as_str()))?;
        let _ = file.write_all(&bytes);
        let resps = http_client.get(format!("{}/cover/{}", mangadex_api::constants::API_URL, cover_to_use.id.hyphenated())).send().await?;
        let mut files = File::create(format!("covers/{}.json", cover_to_use.id.hyphenated()))?;
        files.write_all(&resps.bytes().await?)?;

        vecs.push(format!("{}", cover_to_use.id.hyphenated()));
        println!("downloaded {}", filename.as_str());
    }
    let jsons = serde_json::json!({
        "result" : "ok",
        "id": manga_id,
        "type" : "collection",
        "downloaded" : vecs
    });
    let mut files = File::create(format!("covers/lists/{}.json", manga_id))?;
    files.write_all(jsons.to_string().as_bytes())?;
    
    Ok(jsons)
}

mod path{
    use std::fs::File;
    use std::io::Write;

    use anyhow::Ok;
    use reqwest::Url;
    use uuid::Uuid;
    use mangadex_api::types::RelationshipType;
    use mangadex_api::v5::MangaDexClient;
    use mangadex_api::CDN_URL;
    pub async fn cover_download_by_manga_id(path: &str,manga_id: &str) -> anyhow::Result<()> {
        let client = MangaDexClient::default();
        let manga_id = Uuid::parse_str(manga_id).unwrap();
        let manga = client
            .manga()
            .get()
            .manga_id(&manga_id)
            .build()?
            .send()
            .await?;

        let cover_id = manga
            .data
            .relationships
            .iter()
            .find(|related| related.type_ == RelationshipType::CoverArt)
            .expect("no cover art found for manga")
            .id;
        let cover = client
            .cover()
            .get()
            .cover_id(&cover_id)
            .build()?
            .send()
            .await?;

        // This uses the best quality image.
        // To use smaller, thumbnail-sized images, append any of the following:
        //
        // - .512.jpg
        // - .256.jpg
        //
        // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
        let cover_url = Url::parse(&format!(
                "{}/covers/{}/{}",
                CDN_URL, manga_id, cover.data.attributes.file_name
            ))
            .unwrap();

        let http_client = reqwest::Client::new();

        let res = http_client.get(cover_url).send().await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;
        let filename = cover.data.attributes.file_name;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let mut file = File::create(format!("{}/{}", path,filename.as_str()))?;
        let _ = file.write_all(&bytes);
        println!("downloaded {}", filename.as_str());
        Ok(())
    }

    pub async fn cover_download_quality_by_manga_id(path: &str, manga_id: &str, quality:  u32) -> anyhow::Result<()> {
        let client = MangaDexClient::default();
        let manga_id = Uuid::parse_str(manga_id).unwrap();
        let manga = client
            .manga()
            .get()
            .manga_id(&manga_id)
            .build()?
            .send()
            .await?;

        let cover_id = manga
            .data
            .relationships
            .iter()
            .find(|related| related.type_ == RelationshipType::CoverArt)
            .expect("no cover art found for manga")
            .id;
        let cover = client
            .cover()
            .get()
            .cover_id(&cover_id)
            .build()?
            .send()
            .await?;

        // This uses the best quality image.
        // To use smaller, thumbnail-sized images, append any of the following:
        //
        // - .512.jpg
        // - .256.jpg
        //
        // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
        if quality == 256 || quality == 512 {
            // This uses the best quality image.
            // To use smaller, thumbnail-sized images, append any of the following:
            //
            // - .512.jpg
            // - .256.jpg
            //
            // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
            let cover_url = Url::parse(&format!(
                    "{}/covers/{}/{}",
                    CDN_URL, manga_id, format!("{}.{}.jpg", cover.data.attributes.file_name, quality)
                ))
                .unwrap();

            let http_client = reqwest::Client::new();

            let res = http_client.get(cover_url).send().await?;
            // The data should be streamed rather than downloading the data all at once.
            let bytes = res.bytes().await?;
            let filename = cover.data.attributes.file_name;
            // This is where you would download the file but for this example, we're just printing the raw data.
            let mut file = File::create(format!("{}/{}", path,filename.as_str()))?;
            let _ = file.write_all(&bytes);
            println!("downloaded {}", filename.as_str());
                
        }
        Ok(())
    }

    pub async fn cover_download_by_cover(path: &str, cover_id: &str) -> anyhow::Result<()> {
        let client = MangaDexClient::default();
        let cover = client
            .cover()
            .get()
            .cover_id(&Uuid::parse_str(cover_id).unwrap())
            .build()?
            .send()
            .await?;
        
        let manga_id = cover
            .data
            .relationships
            .iter()
            .find(
                | related 
                | related.type_ == RelationshipType::Manga
            )
            .expect("No manga found")
            .id;
        // This uses the best quality image.
        // To use smaller, thumbnail-sized images, append any of the following:
        //
        // - .512.jpg
        // - .256.jpg
        //
        // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
        let cover_url = Url::parse(&format!(
                "{}/covers/{}/{}",
                CDN_URL, manga_id, cover.data.attributes.file_name
            ))
            .unwrap();

        let http_client = reqwest::Client::new();

        let res = http_client.get(cover_url).send().await?;
        // The data should be streamed rather than downloading the data all at once.
        let bytes = res.bytes().await?;
        let filename = cover.data.attributes.file_name;
        // This is where you would download the file but for this example, we're just printing the raw data.
        let mut file = File::create(format!("{}/{}", path,filename.as_str()))?;
        let _ = file.write_all(&bytes);
        println!("downloaded {}", filename.as_str());
        Ok(())
    }

    pub async fn cover_download_quality_by_cover(path: &str, cover_id: &str, quality:  u32) -> anyhow::Result<()> {
        let client = MangaDexClient::default();
        let cover = client
            .cover()
            .get()
            .cover_id(&Uuid::parse_str(cover_id).unwrap())
            .build()?
            .send()
            .await?;
        
        let manga_id = cover
            .data
            .relationships
            .iter()
            .find(
                | related 
                | related.type_ == RelationshipType::Manga
            )
            .expect("No manga found")
            .id;
        if quality == 256 || quality == 512 {
            // This uses the best quality image.
            // To use smaller, thumbnail-sized images, append any of the following:
            //
            // - .512.jpg
            // - .256.jpg
            //
            // For example, "https://uploads.mangadex.org/covers/8f3e1818-a015-491d-bd81-3addc4d7d56a/4113e972-d228-4172-a885-cb30baffff97.jpg.512.jpg"
            let cover_url = Url::parse(&format!(
                    "{}/covers/{}/{}",
                    CDN_URL, manga_id, format!("{}.{}.jpg", cover.data.attributes.file_name, quality)
                ))
                .unwrap();

            let http_client = reqwest::Client::new();

            let res = http_client.get(cover_url).send().await?;
            // The data should be streamed rather than downloading the data all at once.
            let bytes = res.bytes().await?;
            let filename = cover.data.attributes.file_name;
            // This is where you would download the file but for this example, we're just printing the raw data.
            let mut file = File::create(format!("{}/{}", path,filename.as_str()))?;
            let _ = file.write_all(&bytes);
            println!("downloaded {}", filename.as_str());
                
        }
        Ok(())
    }

    pub async fn all_covers_download_quality_by_manga_id(path: &str, manga_id: &str, limit : u32) -> anyhow::Result<()> {
        let client = MangaDexClient::default();
        let manga_id = Uuid::parse_str(manga_id).unwrap();

        let covers = client
            .cover()
            .list()
            .add_manga_id(&manga_id)
            .limit(limit)
            .build()?
            .send()
            .await?;
        for cover_to_use in covers.data{
            let cover_url = Url::parse(&format!(
                    "{}/covers/{}/{}",
                    CDN_URL, manga_id, cover_to_use.attributes.file_name
                ))
                .unwrap();

            let http_client = reqwest::Client::new();

            let res = http_client.get(cover_url).send().await?;
            // The data should be streamed rather than downloading the data all at once.
            let bytes = res.bytes().await?;
            let filename = cover_to_use.attributes.file_name;
            // This is where you would download the file but for this example, we're just printing the raw data.
            let mut file = File::create(format!("{}/{}", path,filename.as_str()))?;
            let _ = file.write_all(&bytes);
            println!("downloaded {}", filename.as_str());
        }
        Ok(())
    }
}