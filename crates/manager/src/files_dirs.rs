use std::{
    ffi::OsStr,
    fs::File,
    io::BufReader,
    ops::{Deref, DerefMut},
    path::Path,
};

use actix::{Actor, Context, Handler, Message};
use log::error;
use mangadex_api_schema_rust::v5::CoverObject;
use mangadex_api_types_rust::RelationshipType;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

pub mod messages;

use crate::{
    core::ManagerCoreResult,
    history::{service::messages::is_in::IsInMessage, IsIn},
};
use api_core::{data_push::Push, DirsOptions as DirsOptionsCore};

#[derive(Deserialize, Serialize, Clone, Debug)]
pub struct DirsOptions(DirsOptionsCore);

impl From<DirsOptionsCore> for DirsOptions {
    fn from(value: DirsOptionsCore) -> Self {
        Self(value)
    }
}

impl From<DirsOptions> for DirsOptionsCore {
    fn from(value: DirsOptions) -> Self {
        value.0
    }
}

impl Deref for DirsOptions {
    type Target = DirsOptionsCore;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DirsOptions {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl DirsOptions {
    pub fn load_from_path(path: &Path) -> ManagerCoreResult<DirsOptions> {
        Ok(DirsOptionsCore::load_from_path(path).map(Self::from)?)
    }
    pub fn new_from_data_dir<P: AsRef<Path>>(data_dir: P) -> DirsOptions {
        Self::from(DirsOptionsCore::new_from_data_dir(data_dir))
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

#[allow(dead_code)]
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

impl<T> Push<T> for DirsOptions
where
    DirsOptionsCore: Push<T>,
    <DirsOptionsCore as Push<T>>::Error: Into<crate::Error>,
{
    type Error = crate::Error;

    fn push(&mut self, data: T) -> Result<(), Self::Error> {
        self.0.push(data).map_err(|e| e.into())
    }
    fn verify_and_push(&mut self, data: T) -> Result<(), Self::Error> {
        self.0.verify_and_push(data).map(|e| e.into())
    }
}
