use std::collections::HashMap;

use actix_http::header;
use bytes::Bytes;
use http::StatusCode;
use serde::Serialize;
use serde_json::Value;
use tauri::api::http::ResponseType;
use url::Url;

pub struct Response(ResponseType, actix_http::Response<Bytes>, Url);

impl Response {
    /// Get the [`StatusCode`] of this Response.
    pub fn status(&self) -> StatusCode {
        self.1.status()
    }

    /// Get the headers of this Response.
    pub fn headers(&self) -> &header::HeaderMap {
        self.1.headers()
    }

    /// Reads the response as raw bytes.
    pub async fn bytes(self) -> tauri::api::Result<RawResponse> {
        let status = self.status().as_u16();
        let data: &Bytes = self.1.body();
        Ok(RawResponse {
            status,
            data: data.to_vec(),
        })
    }

    /// Reads the response.
    ///
    /// Note that the body is serialized to a [`Value`].
    pub async fn read(self) -> tauri::api::Result<ResponseData> {
        let url = self.2.clone();

        let mut headers = HashMap::new();
        let mut raw_headers = HashMap::new();
        for (name, value) in self.1.headers() {
            headers.insert(
                name.as_str().to_string(),
                String::from_utf8(value.as_bytes().to_vec())?,
            );
            raw_headers.insert(
                name.as_str().to_string(),
                self.1
                    .headers()
                    .get_all(name)
                    .map(|v| String::from_utf8(v.as_bytes().to_vec()).map_err(Into::into))
                    .collect::<tauri::api::Result<Vec<String>>>()?,
            );
        }
        let status = self.1.status().as_u16();

        let data = match self.0 {
            ResponseType::Json => serde_json::from_slice(self.1.body())?,
            ResponseType::Text => Value::String(String::from_utf8(self.1.body().to_vec())?),
            ResponseType::Binary => serde_json::to_value(self.1.body())?,
            _ => Value::Null,
        };

        Ok(ResponseData {
            url,
            status,
            headers,
            raw_headers,
            data,
        })
    }
}

/// A response with raw bytes.
#[non_exhaustive]
#[derive(Debug)]
pub struct RawResponse {
    /// Response status code.
    pub status: u16,
    /// Response bytes.
    pub data: Vec<u8>,
}

/// The response data.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
#[non_exhaustive]
pub struct ResponseData {
    /// Response URL. Useful if it followed redirects.
    pub url: Url,
    /// Response status code.
    pub status: u16,
    /// Response headers.
    pub headers: HashMap<String, String>,
    /// Response raw headers.
    pub raw_headers: HashMap<String, Vec<String>>,
    /// Response data.
    pub data: Value,
}
