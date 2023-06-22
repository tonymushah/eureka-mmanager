use std::collections::HashMap;

use actix_web::HttpRequest;

use crate::utils::query::query_string_to_hash_map;

pub mod get;
pub mod patch;
pub mod delete;
pub mod put;
#[macro_export]
macro_rules! this_api_result {
    ($to_use:expr) => {
        match $to_use {
            Err(e) => {
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : format!("{}", e.to_string())
                });
                return HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body(jsons.to_string());
            }
            Ok(f) => f,
        }
    };
}

#[macro_export]
macro_rules! this_api_option {
    ($to_use:expr, $message:expr) => {
        match $to_use {
            Some(d) => d,
            None => {
                let jsons = serde_json::json!({
                    "result" : "error",
                    "message" : $message
                });
                return HttpResponse::InternalServerError()
                    .content_type(ContentType::json())
                    .body(jsons.to_string());
            }
        }
    };
}

pub fn get_params(request: HttpRequest) -> HashMap<String, String> {
    return match query_string_to_hash_map(request.query_string()) {
        Ok(value) => value,
        Err(error) => {
            println!("{}", error);
            HashMap::new()
        }
    };
}