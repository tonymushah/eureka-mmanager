use std::io::Write;

pub mod files_dirs;
pub mod server_options;


/// it's initialise the dir and all json settings
pub fn initialise_settings_dir() -> anyhow::Result<()>{
    std::fs::create_dir_all("./settings")?;

    // initialise the files-dirs.json file
    let mut file_dir = std::fs::File::create("./settings/files-dirs.json")?;
    // default config
    let file_dir_content = serde_json::json!({
        "data_dir" : "data",
        "chapters" : "chapters",
        "mangas" : "mangas",
        "covers" : "covers"
    });
    file_dir.write_all(file_dir_content.to_string().as_bytes())?;

    // initialise the server-option
    let mut server_option = std::fs::File::create("./settings/server-options.json")?;
    // default config
    let server_option_content = serde_json::json!({
        "hostname" : "127.0.0.1",
        "port" : 8090
    });
    server_option.write_all(server_option_content.to_string().as_bytes())?;

    Ok(())
}

pub fn verify_settings_dir() -> anyhow::Result<String, String> {
    if std::path::Path::new("./settings").exists() == false {
        return Err("the dir settings doesn't exist".to_string());
    }
    if std::path::Path::new("./settings/files-dirs.json").exists() == false {
        return Err("the files-dirs.json in the settings dir doesn't exist".to_string());
    }
    if std::path::Path::new("./settings/server-options.json").exists() == false {
        return Err("the server-options.json in the settings dir doesn't exist".to_string());
    }
    Ok("the dir settings is operationnal".to_string())
}

pub fn initialise_data_dir() -> anyhow::Result<()>{
    let dirs_options : files_dirs::DirsOptions = files_dirs::DirsOptions::new().expect("can't load the dirOption API");
    let dirs_options0 = dirs_options.clone();
    let dirs_options2 = dirs_options.clone();
    let dirs_options3 = dirs_options.clone();
    let dirs_options4 = dirs_options.clone();
    let dirs_options5 = dirs_options.clone();
    std::fs::create_dir_all(dirs_options0.data_dir_add(""))?;
    std::fs::create_dir_all(dirs_options.chapters_add(""))?;
    std::fs::create_dir_all(dirs_options2.covers_add(""))?;
    std::fs::create_dir_all(dirs_options3.mangas_add(""))?;
    std::fs::create_dir_all(dirs_options4.covers_add("lists"))?;
    std::fs::create_dir_all(dirs_options5.covers_add("images"))?;
    Ok(())
}

pub fn verify_data_dir() -> anyhow::Result<String, String>{
    let dirs_options : files_dirs::DirsOptions = files_dirs::DirsOptions::new().expect("can't load the dirOption API");
    let dirs_options0 = dirs_options.clone();
    let dirs_options2 = dirs_options.clone();
    let dirs_options3 = dirs_options.clone();
    if std::path::Path::new(dirs_options0.data_dir_add("").as_str()).exists() == false {
        return Err("the data dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options.chapters_add("").as_str()).exists() == false {
        return Err("the chapters dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options2.covers_add("").as_str()).exists() == false {
        return Err("the covers dir doesn't exist".to_string());
    }
    if std::path::Path::new(dirs_options3.mangas_add("").as_str()).exists() == false {
        return Err("the mangas dir doesn't exist".to_string());
    }
    Ok("the data dir is operational".to_string())
}