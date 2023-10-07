use std::{fs::File, io::BufReader};

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Clone)]
pub struct ServerOptions {
    pub hostname: String,
    pub port: u16,
}

impl ServerOptions {
    pub fn new() -> std::io::Result<ServerOptions> {
        let file = File::open("./settings/server-options.json")?;
        let instance: ServerOptions = serde_json::from_reader(BufReader::new(file))?;
        Ok(instance)
    }
    pub fn get_hostname_port(&self) -> (String, u16) {
        (self.hostname.clone(), self.port)
    }
}
