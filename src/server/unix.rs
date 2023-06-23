use std::os::unix::net::UnixListener;

use actix_web::{HttpServer, dev::Server};
use super::get_actix_app;

pub fn launch_async_server_with_unix_socket(unix_listener : UnixListener) -> std::io::Result<Server>{

    Ok(HttpServer::new(get_actix_app)
        .listen_uds(unix_listener)?
        .run())
}

#[cfg(test)]
mod test_request{
    use anyhow::Ok;
    use reqwest::Client;
    use url::Url;

    #[tokio::test]
    async fn test_request() -> anyhow::Result<()>{
        let url = Url::parse("unix://./socket/mangadex")?;
        println!("{url}");
        
        let client = Client::new();
        let response = client.get(url).send().await?;
        println!("{}", response.text().await?);
        Ok(())
    }
}