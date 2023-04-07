use crate::settings::files_dirs;


pub fn initialise_data_dir() -> anyhow::Result<()> {
    let dirs_options: files_dirs::DirsOptions = match files_dirs::DirsOptions::new() {
        Ok(data) => data,
        Err(_) => {
            return Err(anyhow::Error::msg("can't load the dir options api"));
        }
    };
    std::fs::create_dir_all(dirs_options.data_dir_add(""))?;
    std::fs::create_dir_all(dirs_options.chapters_add(""))?;
    std::fs::create_dir_all(dirs_options.covers_add(""))?;
    std::fs::create_dir_all(dirs_options.mangas_add(""))?;
    std::fs::create_dir_all(dirs_options.covers_add("lists"))?;
    std::fs::create_dir_all(dirs_options.covers_add("images"))?;
    Ok(())
}

pub fn verify_data_dir() -> anyhow::Result<String, String> {
    let dirs_options: files_dirs::DirsOptions = match files_dirs::DirsOptions::new() {
        Ok(data) => data,
        Err(_) => {
            return Err("can't load the file dir api".into());
        }
    };
    if std::path::Path::new(dirs_options.data_dir_add("").as_str()).exists() == false {
        return Err("the data dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options.chapters_add("").as_str()).exists() == false {
        return Err("the chapters dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options.covers_add("").as_str()).exists() == false {
        return Err("the covers dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options.mangas_add("").as_str()).exists() == false {
        return Err("the mangas dir doesn't exist".to_string());
    }
    Ok("the data dir is operational".to_string())
}

