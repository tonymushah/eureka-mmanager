use std::{
    ffi::OsStr,
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use actix::{Actor, Context, Handler, Message};
use log::error;
use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::RelationshipType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

mod chapters;
mod covers;
mod mangas;
pub mod messages;

use crate::{
    core::ManagerCoreResult,
    history::{service::messages::is_in::IsInMessage, IsIn},
    DirsOptionsVerificationError,
};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DirsOptions {
    pub data_dir: PathBuf,
    pub chapters: PathBuf,
    pub mangas: PathBuf,
    pub covers: PathBuf,
    #[serde(default)]
    pub init_dirs_if_not_exists: Option<bool>,
}

impl DirsOptions {
    pub fn load_from_path(path: &Path) -> ManagerCoreResult<DirsOptions> {
        let file = File::open(path)?;
        let instance: DirsOptions = serde_json::from_reader(BufReader::new(file))?;
        Ok(instance)
    }
    pub fn new_from_data_dir<P: AsRef<Path>>(data_dir: P) -> DirsOptions {
        let data_dir = data_dir.as_ref().to_path_buf();
        DirsOptions {
            chapters: data_dir.join("chapters"),
            mangas: data_dir.join("mangas"),
            covers: data_dir.join("covers"),
            init_dirs_if_not_exists: Some(true),
            data_dir,
        }
    }
    pub fn data_dir_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.data_dir.join(path)
    }
    pub fn history_add<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        self.data_dir_add("history").join(path)
    }
    pub fn init_dirs(&self) -> ManagerCoreResult<()> {
        std::fs::create_dir_all(self.data_dir_add(""))?;
        std::fs::create_dir_all(self.history_add(""))?;
        std::fs::create_dir_all(self.chapters_add(""))?;
        std::fs::create_dir_all(self.covers_add(""))?;
        std::fs::create_dir_all(self.mangas_add(""))?;
        std::fs::create_dir_all(self.cover_images_add(""))?;
        Ok(())
    }
    pub fn verify(&self) -> Result<(), DirsOptionsVerificationError> {
        if !self.data_dir.exists() {
            return Err(DirsOptionsVerificationError::DataRoot);
        }
        if !self.history_add("").exists() {
            return Err(DirsOptionsVerificationError::History);
        }
        if !self.chapters.exists() {
            return Err(DirsOptionsVerificationError::Chapters);
        }
        if !self.covers.exists() {
            return Err(DirsOptionsVerificationError::Covers);
        }
        if !self.cover_images_add("").exists() {
            return Err(DirsOptionsVerificationError::CoverImages);
        }
        if !self.mangas.exists() {
            return Err(DirsOptionsVerificationError::Mangas);
        }
        Ok(())
    }
    pub fn verify_and_init(&self) -> ManagerCoreResult<()> {
        if let Ok(()) = self.verify() {
            Ok(())
        } else {
            self.init_dirs()?;
            Ok(())
        }
    }
}

impl Actor for DirsOptions {
    type Context = Context<Self>;
    fn started(&mut self, _ctx: &mut Self::Context) {
        if let Err(e) = self.verify_and_init() {
            error!("{:#?}", e);
        }
    }
}

impl DirsOptions {
    fn is_chapter_here(&self, id: Uuid) -> bool {
        let chapter = self.chapters_add(format!("{id}"));
        chapter.exists() && chapter.join("data.json").exists()
    }
    fn re_ca(&self, id: Uuid) -> ManagerCoreResult<CoverObject> {
        use mangadex_api_schema_rust::ApiData;
        use serde_json::from_reader;
        let file = BufReader::new(File::open(self.covers_add(format!("{id}.json")))?);
        let d: ApiData<CoverObject> = from_reader(file)?;
        Ok(d.data)
    }
    fn cover_art(&self, id: Uuid) -> bool {
        let cover: ManagerCoreResult<CoverObject> = self.re_ca(id);
        if let Ok(p) = cover.map(|c| self.cover_images_add(c.attributes.file_name)) {
            p.exists()
        } else {
            false
        }
    }

    fn manga(&self, id: Uuid) -> bool {
        self.mangas_add(format!("{id}.json")).exists()
    }
}

impl IsIn<(Uuid, RelationshipType)> for DirsOptions {
    type Output = bool;
    fn is_in(&self, (id, rel): (Uuid, RelationshipType)) -> Self::Output {
        match rel {
            RelationshipType::Chapter => self.is_chapter_here(id),
            RelationshipType::Manga => self.manga(id),
            RelationshipType::CoverArt => self.cover_art(id),
            _ => false,
        }
    }
}

impl Handler<IsInMessage> for DirsOptions {
    type Result = <IsInMessage as Message>::Result;
    fn handle(&mut self, msg: IsInMessage, _ctx: &mut Self::Context) -> Self::Result {
        self.is_in((msg.id, msg.data_type))
    }
}

pub(crate) trait FileExtension {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool;
    fn is_image(&self) -> bool {
        self.has_extension(&[
            "jpg", "jpeg", "jpe", "jif", "jfif", "jfi", "png", "gif", "webp", "tiff", "tif", "svg",
            "svgz",
        ])
    }
    fn is_json(&self) -> bool {
        self.has_extension(&["json"])
    }
    fn is_cbor(&self) -> bool {
        self.has_extension(&["cbor"])
    }
}

impl<P: AsRef<Path>> FileExtension for P {
    fn has_extension<S: AsRef<str>>(&self, extensions: &[S]) -> bool {
        if let Some(extension) = self.as_ref().extension().and_then(OsStr::to_str) {
            return extensions
                .iter()
                .any(|x| x.as_ref().eq_ignore_ascii_case(extension));
        }

        false
    }
}
