use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerOptions{
    pub hostname : String,
    pub port : u16
}

impl ServerOptions{
    pub fn new() -> std::io::Result<ServerOptions>{
        let instance: ServerOptions = serde_json::from_str(std::fs::read_to_string("./settings/server-options.json")?.as_str())?;
        Ok(instance)
    }
    pub fn get_hostname_port(&self) -> (String, u16) {
        (self.hostname.clone(), self.port)
    }
}