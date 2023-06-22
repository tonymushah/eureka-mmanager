use std::fs::File;
use std::io::{Write};

use mangadex_api::HttpClientRef;

use crate::download::cover::cover_download_by_manga_id;
use crate::settings::files_dirs::DirsOptions;
use crate::utils::send_request;

pub async fn download_manga(client : HttpClientRef, mangaid: uuid::Uuid) -> Result<(), std::io::Error>{
    let id = format!("{}", mangaid);
    let http_client = client.lock().await.client.clone();
    let resp = match send_request(http_client.get(format!("{}/manga/{}?includes%5B%5D=author&includes%5B%5D=cover_art&includes%5B%5D=manga&includes%5B%5D=artist&includes%5B%5D=scanlation_group", mangadex_api::constants::API_URL, id)), 5).await {
        Ok(data) => data,
        Err(error) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()));
        }
    } ;
    let mut file = (File::create(
        match DirsOptions::new() {
            Ok(data) => data,
            Err(error) => {
                return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()));
            }
        }
            .mangas_add(format!("{}.json", id).as_str())
    ))?;
    file.write_all(&(match resp.bytes().await {
        Ok(data) => data,
        Err(error) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()));
        }
    }))?;
    match cover_download_by_manga_id(id.to_string().as_str(), client.clone()).await {
        Ok(_) => (),
        Err(error ) => {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, error.to_string()));
        }
    };
    Ok(())
}