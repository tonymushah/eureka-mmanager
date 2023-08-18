use std::fs::File;
use std::io::Write;

use mangadex_api::HttpClientRef;
use mangadex_api_schema_rust::v5::MangaAttributes;
use mangadex_api_schema_rust::{ApiObject, ApiData};

use crate::core::ManagerCoreResult;
use crate::download::cover::cover_download_by_manga_id;
use crate::settings::files_dirs::DirsOptions;
use crate::utils::send_request;

/// download the manga with the specified id 
pub async fn download_manga(client : HttpClientRef, mangaid: uuid::Uuid) -> ManagerCoreResult<()>{
    let id = format!("{}", mangaid);
    let http_client = client.lock().await.client.clone();
    let resp = send_request(http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, id)), 5).await?;
    println!("{:#?}", resp);
    let bytes = resp.bytes().await?;
    let bytes_string = String::from_utf8(bytes.to_vec())?;
    println!("{}", bytes_string);
    serde_json::from_str::<ApiData<ApiObject<MangaAttributes>>>(bytes_string.as_str())?;
    let mut file = (File::create(
        DirsOptions::new()?
            .mangas_add(format!("{}.json", id).as_str())
    ))?;
    file.write_all(&bytes)?;
    cover_download_by_manga_id(id.to_string().as_str(), client.clone()).await?;
    Ok(())
}