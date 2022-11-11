use std::fs::File;
use std::io::{Write, ErrorKind};
//use anyhow::Ok;
use mangadex_api_schema::v5::{
    ChapterAttributes
};
use mangadex_api_schema::{
    ApiData, 
    ApiObject
};
use mangadex_api_types::RelationshipType;

use crate::settings::files_dirs::DirsOptions;

pub async fn update_chap_by_id(id: String) -> anyhow::Result<serde_json::Value> {
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add(format!("{}/data.json", id).as_str());

        let http_client = reqwest::Client::new();
        let get_chapter = http_client
            .get(
                format!("{}/chapter/{}?includes%5B0%5D=scanlation_group&includes%5B1%5D=manga&includes%5B2%5D=user", 
                    mangadex_api::constants::API_URL, 
                    id
                )
            )
            .send()
            .await
            .expect("Can't rend request");
        
            let bytes_ = get_chapter.bytes()
            .await
            .expect("error on exporting to bytes");
        
            let mut chapter_data = File::create((path).as_str())
            .expect("Error on creating file");

        chapter_data.write_all(&bytes_).unwrap();
        
        let jsons = std::fs::read_to_string(path.as_str()).expect("Cannot open file");
        
        Ok(serde_json::from_str(jsons.as_str()).expect("can't covert to json"))
}

pub async fn is_chap_related_to_manga(chap_id: String, manga_id: String) -> anyhow::Result<bool>{
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add(format!("{}/data.json", chap_id).as_str());
    let chapter : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_str(&std::fs::read_to_string(path.as_str())
        .expect(format!("can't find or read file {}", path).as_str()))
        .expect("Can't covert to json");
    let mut is = false;
    for relas in chapter.data.relationships{
        if relas.type_ == RelationshipType::Manga && relas.id.hyphenated().to_string() == manga_id{
            is = true;
        }
    }
    Ok(is)
}

pub async fn find_all_downloades_by_manga_id(manga_id: String) -> anyhow::Result<serde_json::Value> {
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add("");
        let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let to_use = files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string();
            let to_insert = to_use.clone();
            if is_chap_related_to_manga(to_use, manga_id.clone()).await.expect("Error on validating") == true {
                vecs.push(to_insert);
            }
        }
    Ok(serde_json::json!(vecs))
}

pub async fn find_and_delete_all_downloades_by_manga_id(manga_id: String) -> anyhow::Result<serde_json::Value> {
    let files_dirs : DirsOptions = DirsOptions::new()?;
    let path = files_dirs.chapters_add("");
        let list_dir = std::fs::read_dir(path.as_str()).expect("Cannot open file");
        let mut vecs: Vec<String> = Vec::new();
        for files in list_dir {
            let to_use = files.expect("can't open file").file_name().to_str().expect("can't reconize file").to_string();
            let to_insert = to_use.clone();
            let to_remove = to_use.clone();
            if is_chap_related_to_manga(to_use, manga_id.clone()).await.expect("Error on validating") == true {
                vecs.push(to_insert);
                std::fs::remove_dir_all(
                    DirsOptions::new()?
                        .chapters_add(to_remove.as_str())
                )?
            }
        }
    Ok(serde_json::json!(vecs))
}

pub async fn patch_manga_by_chapter(chap_id: String) -> anyhow::Result<serde_json::Value> {
    let path = DirsOptions::new()?.chapters_add(format!("{}/data.json", chap_id).as_str());
    let chapter : ApiData<ApiObject<ChapterAttributes>> = serde_json::from_str(&std::fs::read_to_string(path.as_str())
        .expect(format!("can't find or read file {}", path).as_str()))
        .expect("Can't covert to json");
    let manga_id = chapter
        .data
        .relationships
        .iter()
        .find(|related| related.type_ == RelationshipType::Manga)
        .expect(format!("can't find manga in the chapter {}", chap_id).as_str())
        .id;
    let http_client = reqwest::Client::new();
    let resp = http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, manga_id.hyphenated())).send().await.unwrap();
    let mut file = File::create(
        DirsOptions::new()?
                .mangas_add(format!("{}.json", manga_id.hyphenated()).as_str())
                .as_str())
        .unwrap();

    file.write_all(&(resp.bytes().await.unwrap())).unwrap();
    let jsons = serde_json::json!({
            "result" : "ok",
            "type" : "manga",
            "id" : manga_id.hyphenated()
        });
    println!("downloaded {}.json", manga_id.hyphenated());
    Ok(jsons)
}

pub async fn send_request(to_use: reqwest::RequestBuilder, tries_limits: u16) -> Result<reqwest::Response, std::io::Error>{
    let mut tries = 0;
    //let mut to_return : reqwest::Response;
    while tries < tries_limits {
        let resp = to_use.try_clone().unwrap().send().await;
        if resp.is_err() == true {
            tries = tries + 1;
        }else{
            return Ok(resp.unwrap());
        }
    }
    Err(std::io::Error::new(ErrorKind::Other, "All tries failed to applies your request"))
}