use std::io::{BufWriter, Write};

/// it's initialise the dir and all json settings
pub fn initialise_settings_dir() -> anyhow::Result<()> {
    std::fs::create_dir_all("./settings")?;

    // initialise the files-dirs.json file
    {
        // default config
        let file_dir_content = serde_json::json!({
            "data_dir" : "data",
            "chapters" : "chapters",
            "mangas" : "mangas",
            "covers" : "covers"
        });
        let file_dir = std::fs::File::create("./settings/files-dirs.json")?;
        let mut writer = BufWriter::new(file_dir);
        writer.write_all(file_dir_content.to_string().as_bytes())?;
        writer.flush()?;
    }
    // initialise the server-option
    {
        // default config
        let server_option_content = serde_json::json!({
            "hostname" : "127.0.0.1",
            "port" : 8145
        });
        let server_option = std::fs::File::create("./settings/server-options.json")?;
        let mut writer = BufWriter::new(server_option);
        writer.write_all(server_option_content.to_string().as_bytes())?;
        writer.flush()?
    }
    Ok(())
}

pub fn verify_settings_dir() -> anyhow::Result<String, String> {
    if !std::path::Path::new("./settings").exists() {
        return Err("the dir settings doesn't exist".to_string());
    }
    if !std::path::Path::new("./settings/files-dirs.json").exists() {
        return Err("the files-dirs.json in the settings dir doesn't exist".to_string());
    }
    if !std::path::Path::new("./settings/server-options.json").exists() {
        return Err("the server-options.json in the settings dir doesn't exist".to_string());
    }
    Ok("the dir settings is operationnal".to_string())
}
