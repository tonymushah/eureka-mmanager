use log::info;
use std::{io::ErrorKind};

pub mod chapter;
pub mod collection;
pub mod cover;
pub mod manga;
pub mod manga_aggregate;
pub mod query;

pub async fn send_request(
    to_use_arg: reqwest::RequestBuilder,
    tries_limits: u16,
) -> Result<reqwest::Response, std::io::Error> {
    let mut tries = 0;
    let to_use = to_use_arg;
    //let mut to_return : reqwest::Response;
    while tries < tries_limits {
        let resp = match to_use.try_clone() {
            None => {
                return Err(std::io::Error::new(
                    ErrorKind::Other,
                    "can't clone the request",
                ));
            }
            Some(data) => data,
        }
        .send()
        .await;
        match resp {
            Err(_) => {
                tries += 1;
                info!("tries {}", tries);
            }
            core::result::Result::Ok(data) => {
                return core::result::Result::Ok(data);
            }
        }
    }
    Err(std::io::Error::new(
        ErrorKind::Other,
        "All tries failed to applies your request",
    ))
}
