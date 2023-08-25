use std::collections::HashMap;

use actix_web::HttpRequest;

use crate::utils::query::query_string_to_hash_map;

pub mod get;
pub mod patch;
pub mod delete;
pub mod put;

pub fn get_params(request: HttpRequest) -> HashMap<String, String> {
    return match query_string_to_hash_map(request.query_string()) {
        Ok(value) => value,
        Err(_) => {
            HashMap::new()
        }
    };
}

pub trait DefaultOffsetLimit<'a>: serde::Deserialize<'a> {
    type OffsetOutput;
    type LimitOutput;
    fn default_offset() -> Self::OffsetOutput;
    fn default_limit() -> Self::LimitOutput;
}