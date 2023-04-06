use std::{fs::File, path::Path};

use mangadex_api_schema::{v5::CoverAttributes, ApiObject, ApiData};

use crate::settings::files_dirs::DirsOptions;

pub fn is_cover_image_there(cover_id : String) -> Result<bool, std::io::Error>{
    if cover_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.covers_add(format!("{}.json", cover_id).as_str());
        let cover_data : ApiData<ApiObject<CoverAttributes>> = serde_json::from_reader(File::open(path)?)?;
        let cover_file_name = cover_data.data.attributes.file_name;
        let cover_file_name_path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.covers_add(format!("images/{}", cover_file_name).as_str());
        if Path::new(cover_file_name_path.as_str()).exists() {
            std::io::Result::Ok(true)
        }else{
            std::io::Result::Ok(false)
        }
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "the cover_id should'nt be empty"));
    }
}

pub fn is_cover_there(cover_id : String) -> Result<bool, std::io::Error>{
    if cover_id.is_empty() == false {
        let path = match DirsOptions::new(){
            core::result::Result::Ok(data) => data,
            Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
        }.covers_add(format!("{}.json", cover_id).as_str());
        if Path::new(path.as_str()).exists() {
            return is_cover_image_there(cover_id);
        }else{
            std::io::Result::Ok(false)
        }
    }else{
        return Err(std::io::Error::new(std::io::ErrorKind::Other, "the cover_id should'nt be empty"));
    }
}


pub fn get_cover_data(cover_id : String) -> Result<ApiData<ApiObject<CoverAttributes>>, std::io::Error>{
    let cover_id_clone = cover_id.clone();
    match is_cover_there(cover_id) {
        core::result::Result::Ok(is_there) => {
            if is_there == true{
                let path = match DirsOptions::new(){
                    core::result::Result::Ok(data) => data,
                    Err(e) => return Err(std::io::Error::new(std::io::ErrorKind::Other, e.to_string()))
                }.covers_add(format!("{}.json", cover_id_clone).as_str());
                let data : ApiData<ApiObject<CoverAttributes>> = serde_json::from_str(std::fs::read_to_string(path)?.as_str())?;
                core::result::Result::Ok(data)
            }else{
                Err(std::io::Error::new(std::io::ErrorKind::NotFound, "Cover not found"))
            }
        },
        Err(error) => Err(error)
    }
}
