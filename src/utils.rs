use log::info;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::File,
    io::{BufReader, BufWriter, ErrorKind},
    path::PathBuf,
};

use crate::ManagerCoreResult;

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

pub trait ExtractData {
    type Output: DeserializeOwned;
    type Input: Serialize;
    fn get_file_path(&self) -> ManagerCoreResult<PathBuf>;
    fn get_file(&self) -> ManagerCoreResult<File> {
        Ok(File::open(self.get_file_path()?)?)
    }
    fn get_file_create(&self) -> ManagerCoreResult<File> {
        Ok(File::create(self.get_file_path()?)?)
    }
    fn get_buf_reader(&self) -> ManagerCoreResult<BufReader<File>> {
        Ok(BufReader::new(self.get_file()?))
    }
    fn get_buf_writer(&self) -> ManagerCoreResult<BufWriter<File>> {
        Ok(BufWriter::new(self.get_file_create()?))
    }
    fn get_data(&self) -> ManagerCoreResult<Self::Output> {
        Ok(serde_json::from_reader(self.get_buf_reader()?)?)
    }
    fn update(&self, input: Self::Input) -> ManagerCoreResult<()>;
    fn delete(&self) -> ManagerCoreResult<()>;
    fn is_there(&self) -> bool {
        self.get_data().is_ok()
    }
}
