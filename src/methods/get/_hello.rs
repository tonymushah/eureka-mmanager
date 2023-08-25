use actix_web::http::header::ContentType;
use actix_web::{get, HttpResponse, Responder};

/// try if the app is ok
/// # How to use
/// {app deployed url}/
#[get("/")]
pub async fn hello(/*request: HttpRequest*/) -> impl Responder {
    HttpResponse::Ok().content_type(ContentType::json()).body(
        serde_json::json!({
            "result" : "ok",
            "message" : "The mangadex desktop api works !!"
        })
        .to_string(),
    )
}
