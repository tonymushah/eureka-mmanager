use std::{
    fs::File,
    io::{BufWriter, Write},
};

use mangadex_api_schema_rust::v5::MangaObject;

use crate::DirsOptions;

use super::Push;

impl Push<MangaObject> for DirsOptions {
    fn push(&mut self, data: MangaObject) -> crate::ManagerCoreResult<()> {
        let mut file = BufWriter::new(File::create(self.mangas_add(format!("{}.json", data.id)))?);
        serde_json::to_writer(&mut file, &data)?;
        file.flush()?;
        Ok(())
    }
}

impl Push<Vec<MangaObject>> for DirsOptions {
    fn push(&mut self, data: Vec<MangaObject>) -> crate::ManagerCoreResult<()> {
        for manga in data {
            self.push(manga)?;
        }
        Ok(())
    }
}
