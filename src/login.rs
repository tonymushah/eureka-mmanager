use mangadex_api::v5::MangaDexClient;
use mangadex_api::types::Username;
use mangadex_api::types::Password;
use serde_json::json;

async fn login_default(username: &str, password: &str) -> anyhow::Result<String>{
    let client : MangaDexClient = MangaDexClient::default();
    let logged_user = client
        .auth()
        .login()
        .username(Username::parse(username).unwrap())
        .password(Password::parse(password).unwrap())
        .build()?
        .send()
        .await?;

    Ok(format!("{}", json!({
        "session" : logged_user.token.session, 
        "refresh" : logged_user.token.refresh
    })))
}

async fn login_email(email: &str, password: &str) -> anyhow::Result<String>{
    let client : MangaDexClient = MangaDexClient::default();
    let logged_user = client
        .auth()
        .login()
        .email(email)
        .password(Password::parse(password).unwrap())
        .build()?
        .send()
        .await?;

    Ok(format!("{}", json!({
        "session" : logged_user.token.session, 
        "refresh" : logged_user.token.refresh
    })))
}