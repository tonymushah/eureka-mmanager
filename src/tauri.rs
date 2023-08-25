use actix_http::Request;
use actix_service::IntoServiceFactory;
use actix_service::Service;
use actix_web::dev::{AppConfig, ServiceFactory, ServiceResponse};

use crate::server::get_actix_app;

use self::bindings::request::HttpRequestBuilder;
pub mod bindings;
pub(crate) async fn try_init_service<R, S, B, E>(
    app: R,
) -> Result<impl Service<Request, Response = ServiceResponse<B>, Error = E>, S::InitError>
where
    R: IntoServiceFactory<S, Request>,
    S: ServiceFactory<Request, Config = AppConfig, Response = ServiceResponse<B>, Error = E>,
    S::InitError: std::fmt::Debug,
{
    let srv = app.into_factory();
    srv.new_service(AppConfig::default()).await
}

pub async fn get_app_service<T>() -> Result<
    impl actix_service::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<impl actix_web::body::MessageBody>,
        Error = actix_web::Error,
    >,
    (),
> {
    try_init_service(get_actix_app()).await
}

#[derive(Default)]
pub struct ActixTauriState<T, I>
where
    T: actix_service::Service<
        actix_http::Request,
        Response = actix_web::dev::ServiceResponse<I>,
        Error = actix_web::Error,
    >,
    I: actix_web::body::MessageBody,
{
    #[allow(dead_code)]
    s: tokio::sync::Mutex<Option<T>>,
}
// remember to call `.manage(MyState::default())`
#[tauri::command]
#[allow(dead_code)]
async fn command_name<T, I>(
    _request: HttpRequestBuilder,
    _state: tauri::State<'_, ActixTauriState<T, I>>,
) -> Result<(), String>
where
    T: actix_service::Service<
            actix_http::Request,
            Response = actix_web::dev::ServiceResponse<I>,
            Error = actix_web::Error,
        > + Send,
    I: actix_web::body::MessageBody,
{
    // let service : T = get_app_service().await.unwrap();
    // state.s.lock().await.replace(service);
    todo!()
}

mod private {
    use std::str::FromStr;

    use super::bindings::request::{Body, HttpRequestBuilder};
    use actix_http::{h1::Payload, Method, Request, Uri};
    use bytes::Bytes;
    use http::{HeaderName, HeaderValue};
    pub trait TryFrom<T>: Sized {
        type Error;

        fn try_from(value: T) -> Result<Self, Self::Error>;
    }
    pub trait From<T>: Sized {
        fn from(data: T) -> Self;
    }
    impl From<HttpRequestBuilder> for Request {
        fn from(data: HttpRequestBuilder) -> Self {
            let mut returns: Self = if let Some(body) = data.body {
                let bytes: Bytes = match body {
                    Body::Form(_) => Bytes::default(),
                    Body::Json(h) => Bytes::from(h.to_string()),
                    Body::Text(t) => Bytes::from(t),
                    Body::Bytes(b) => Bytes::from(b),
                };
                let (mut sender, payload) = Payload::create(false);
                sender.feed_data(bytes);
                sender.feed_eof();
                Self::with_payload(actix_http::Payload::H1 { payload })
            } else {
                Self::new()
            };
            let mut ret_head = returns.head_mut();
            ret_head.method = match Method::from_str(data.method.as_str()) {
                Ok(d) => d,
                Err(_) => Default::default(),
            };
            if let Some(header) = data.headers {
                for (key_, value) in header.0 {
                    if let Some(key) = key_ {
                        ret_head.headers.insert(key, value);
                    }
                }
            }
            ret_head.method = match Method::from_str(data.method.as_str()) {
                Ok(d) => d,
                Err(_) => {
                    ret_head.headers_mut().insert(
                        HeaderName::from_static("invalid-method"),
                        HeaderValue::from_static("true"),
                    );
                    Default::default()
                }
            };
            let mut url = data.url.clone();
            if let Some(query) = data.query {
                for (key, value) in query {
                    url.query_pairs_mut()
                        .append_pair(key.as_str(), value.as_str());
                }
            }
            ret_head.uri = match url.as_str().parse::<Uri>() {
                Ok(uri) => uri,
                Err(_) => {
                    ret_head.headers_mut().insert(
                        HeaderName::from_static("invalid-uri"),
                        HeaderValue::from_static("true"),
                    );
                    Uri::default()
                }
            };
            returns
        }
    }
}
