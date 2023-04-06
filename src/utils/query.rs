use std::collections::HashMap;
use std::hash::Hash;

pub fn query_string_to_hash_map(to_use: &str) -> Result<HashMap<String, String>, std::io::Error>{
    let mut to_return : HashMap<String, String> = HashMap::new();
    let query_part: Vec<&str> = to_use.split('&').collect();
    for parts in query_part {
        let query_part_parsed = match parts.split_once("=") {
            None => continue,
            Some(value) => value
        };
        to_return.insert(query_part_parsed.0.to_string(), query_part_parsed.1.to_string());
    }
    std::io::Result::Ok(to_return)
}

pub fn get_query_hash_value_or_else<T>(to_use: &HashMap<T, T>, to_get: T, or_else: T) -> T
    where T : std::cmp::Eq,
    T : Hash,
    T : Clone
{
    match to_use.get(&to_get) {
        Some(data) => data.clone(),
        None => or_else
    }
}
