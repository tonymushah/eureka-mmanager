use anyhow::Ok;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct ServerOptions{
    pub hostname : String,
    pub port : u16
}

impl ServerOptions{
    pub fn new() -> anyhow::Result<ServerOptions>{
        let instance: ServerOptions = serde_json::from_str(std::fs::read_to_string("./settings/server-options.json")?.as_str())?;
        Ok(instance)
    }
}