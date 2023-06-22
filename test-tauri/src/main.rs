// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::io::Write;

use tauri::http::{Response, status::StatusCode, header};

fn main() {
  tauri::Builder::default()
    .register_uri_scheme_protocol("mangadex", |_app, _req| {
      let mut res = Response::default();
      res.headers_mut().insert("Access-Control-Allow-Origin", header::HeaderValue::from_static("*"));
      res.body_mut().write_all(serde_json::json!({
        "result" : "ok"
      }).to_string().as_bytes())?;
      res.set_status(StatusCode::ACCEPTED);
      Ok(res)
    })
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
