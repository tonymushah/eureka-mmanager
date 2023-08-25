use std::{collections::HashMap, time::Duration};

use http::{header, HeaderName, HeaderValue};
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use tauri::api::http::FormPart;
use tauri::api::http::ResponseType;
use url::Url;

#[derive(Deserialize)]
#[serde(untagged)]
enum SerdeDuration {
    Seconds(u64),
    Duration(Duration),
}

/// A set of HTTP headers.
#[derive(Debug, Default)]
pub struct HeaderMap(pub header::HeaderMap);

impl<'de> Deserialize<'de> for HeaderMap {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map = HashMap::<String, String>::deserialize(deserializer)?;
        let mut headers = header::HeaderMap::default();
        for (key, value) in map {
            if let (Ok(key), Ok(value)) = (
                header::HeaderName::from_bytes(key.as_bytes()),
                header::HeaderValue::from_str(&value),
            ) {
                headers.insert(key, value);
            } else {
                return Err(serde::de::Error::custom(format!(
                    "invalid header `{key}` `{value}`"
                )));
            }
        }
        Ok(Self(headers))
    }
}

#[derive(Debug, Deserialize)]
pub struct FormBody(pub(crate) HashMap<String, FormPart>);

impl FormBody {
    /// Creates a new form body.
    pub fn new(data: HashMap<String, FormPart>) -> Self {
        Self(data)
    }
}

/// A body for the request.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", content = "payload")]
#[non_exhaustive]
pub enum Body {
    /// A form body.
    Form(FormBody),
    /// A JSON body.
    Json(Value),
    /// A text string body.
    Text(String),
    /// A byte array body.
    Bytes(Vec<u8>),
}

fn deserialize_duration<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Duration>, D::Error> {
    if let Some(duration) = Option::<SerdeDuration>::deserialize(deserializer)? {
        Ok(Some(match duration {
            SerdeDuration::Seconds(s) => Duration::from_secs(s),
            SerdeDuration::Duration(d) => d,
        }))
    } else {
        Ok(None)
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HttpRequestBuilder {
    /// The request method (GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS, CONNECT or TRACE)
    pub method: String,
    /// The request URL
    pub url: Url,
    /// The request query params
    pub query: Option<HashMap<String, String>>,
    /// The request headers
    pub headers: Option<HeaderMap>,
    /// The request body
    pub body: Option<Body>,
    /// Timeout for the whole request
    #[serde(deserialize_with = "deserialize_duration", default)]
    pub timeout: Option<Duration>,
    /// The response type (defaults to Json)
    pub response_type: Option<ResponseType>,
}

impl HttpRequestBuilder {
    /// Initializes a new instance of the HttpRequestrequest_builder.
    pub fn new(method: impl Into<String>, url: impl AsRef<str>) -> tauri::api::Result<Self> {
        Ok(Self {
            method: method.into(),
            url: Url::parse(url.as_ref())?,
            query: None,
            headers: None,
            body: None,
            timeout: None,
            response_type: None,
        })
    }

    /// Sets the request parameters.
    #[must_use]
    pub fn query(mut self, query: HashMap<String, String>) -> Self {
        self.query = Some(query);
        self
    }

    /// Adds a header.
    pub fn header<K, V>(mut self, key: K, value: V) -> tauri::api::Result<Self>
    where
        HeaderName: TryFrom<K>,
        <HeaderName as TryFrom<K>>::Error: Into<http::Error>,
        HeaderValue: TryFrom<V>,
        <HeaderValue as TryFrom<V>>::Error: Into<http::Error>,
    {
        let key: Result<HeaderName, http::Error> = key.try_into().map_err(Into::into);
        let value: Result<HeaderValue, http::Error> = value.try_into().map_err(Into::into);
        self.headers
            .get_or_insert_with(Default::default)
            .0
            .insert(key?, value?);
        Ok(self)
    }

    /// Sets the request headers.
    #[must_use]
    pub fn headers(mut self, headers: header::HeaderMap) -> Self {
        self.headers.replace(HeaderMap(headers));
        self
    }

    /// Sets the request body.
    #[must_use]
    pub fn body(mut self, body: Body) -> Self {
        self.body = Some(body);
        self
    }

    /// Sets the general request timeout.
    #[must_use]
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout.replace(timeout);
        self
    }

    /// Sets the type of the response. Interferes with the way we read the response.
    #[must_use]
    pub fn response_type(mut self, response_type: ResponseType) -> Self {
        self.response_type = Some(response_type);
        self
    }
}
