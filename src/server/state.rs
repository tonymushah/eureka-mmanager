use actix_web::{get, web, App, HttpServer, Responder};
use std::{
    cell::Cell,
    sync::atomic::{AtomicUsize, Ordering},
    sync::Arc,
};

use std::{collections::HashMap, io::Write, sync::Mutex};

use mangadex_api_types::RelationshipType;

use crate::settings::{file_history::HistoryWFile, files_dirs::DirsOptions};


#[derive(Clone)]
pub struct AppState{
    pub(crate) dir_option : Arc<DirsOptions>,
    pub(crate) history : Arc<HashMap<RelationshipType, HistoryWFile>>
}
