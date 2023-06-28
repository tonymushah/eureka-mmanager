use std::{fs::File, path::Path};

use async_stream::stream;
use futures::Stream;
use mangadex_api_schema_rust::{v5::CoverAttributes, ApiData, ApiObject};

use crate::settings::files_dirs::DirsOptions;

pub fn is_cover_image_there(cover_id: String) -> Result<bool, std::io::Error> {
    if !cover_id.is_empty() {
        let path = match DirsOptions::new() {
            core::result::Result::Ok(data) => data,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            }
        }
        .covers_add(format!("{}.json", cover_id).as_str());
        let cover_data: ApiData<ApiObject<CoverAttributes>> =
            serde_json::from_reader(File::open(path)?)?;
        let cover_file_name = cover_data.data.attributes.file_name;
        let cover_file_name_path = match DirsOptions::new() {
            core::result::Result::Ok(data) => data,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            }
        }
        .covers_add(format!("images/{}", cover_file_name).as_str());
        if Path::new(cover_file_name_path.as_str()).exists() {
            std::io::Result::Ok(true)
        } else {
            std::io::Result::Ok(false)
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "the cover_id should'nt be empty",
        ))
    }
}

pub fn is_cover_there(cover_id: String) -> Result<bool, std::io::Error> {
    if !cover_id.is_empty() {
        let path = match DirsOptions::new() {
            core::result::Result::Ok(data) => data,
            Err(e) => {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    e.to_string(),
                ))
            }
        }
        .covers_add(format!("{}.json", cover_id).as_str());
        if Path::new(path.as_str()).exists() {
            is_cover_image_there(cover_id)
        } else {
            std::io::Result::Ok(false)
        }
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "the cover_id should'nt be empty",
        ))
    }
}

pub fn get_cover_data(
    cover_id: String,
) -> Result<ApiData<ApiObject<CoverAttributes>>, std::io::Error> {
    let cover_id_clone = cover_id.clone();
    match is_cover_there(cover_id) {
        core::result::Result::Ok(is_there) => {
            if is_there {
                let path = match DirsOptions::new() {
                    core::result::Result::Ok(data) => data,
                    Err(e) => {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        ))
                    }
                }
                .covers_add(format!("{}.json", cover_id_clone).as_str());
                let data: ApiData<ApiObject<CoverAttributes>> =
                    serde_json::from_str(std::fs::read_to_string(path)?.as_str())?;
                core::result::Result::Ok(data)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::NotFound,
                    "Cover not found",
                ))
            }
        }
        Err(error) => Err(error),
    }
}

pub fn get_all_cover() -> Result<impl Stream<Item = String>, std::io::Error> {
    let file_dirs = match DirsOptions::new() {
        core::result::Result::Ok(data) => data,
        Err(error) => {
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                error.to_string(),
            ))
        }
    };
    let path = file_dirs.covers_add("");
    if Path::new(path.as_str()).exists() {
        let list_dir = (std::fs::read_dir(path.as_str()))?;
        Ok(stream! {
            for files in list_dir {
                if let core::result::Result::Ok(file_) = files {
                    if let core::result::Result::Ok(metadata) = file_.metadata() {
                        if metadata.is_file() {
                            if let Some(data) = file_.file_name().to_str() {
                                yield data.to_string().replace(".json", "");
                            }
                        }
                    }
                }
            }
        })
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "can't find the cover directory",
        ))
    }
}
