use std::{
    fs::{create_dir_all, File},
    io::BufWriter,
};

use mangadex_api_schema_rust::v5::ChapterObject;

use crate::ManagerCoreResult;

use super::{DataPush, Push};

impl<'a> Push<ChapterObject> for DataPush<'a> {
    fn push(&mut self, data: ChapterObject) -> crate::ManagerCoreResult<()> {
        let chapter_path = self.chapters_id_add(data.id);
        create_dir_all(&chapter_path)?;
        let mut file = BufWriter::new(File::create(chapter_path.join("data.json"))?);
        serde_json::to_writer(&mut file, &data)?;
        Ok(())
    }
}

impl<'a> Push<Vec<ChapterObject>> for DataPush<'a> {
    fn push(&mut self, data: Vec<ChapterObject>) -> ManagerCoreResult<()> {
        for item in data {
            self.push(item)?;
        }
        Ok(())
    }
}
