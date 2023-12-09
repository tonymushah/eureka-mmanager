#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        "User-Agent",
        reqwest::header::HeaderValue::from_static("special-eureka-manager/0.4.0"),
    );
    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    let client = mangadex_api::v5::MangaDexClient::new(client);

    let random_manga = client
        .manga()
        .id(uuid::Uuid::parse_str(
            "63fb6354-0ace-4f74-b8b8-af1be314f245",
        )?)
        .get()
        .send()
        .await?;

    println!("{:#?}", random_manga);

    Ok(())
}
