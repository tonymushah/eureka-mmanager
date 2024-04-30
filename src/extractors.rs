use std::{
    fs::File,
    io::{BufReader, BufWriter},
    path::PathBuf,
};

use serde::{de::DeserializeOwned, Serialize};

use crate::ManagerCoreResult;

pub trait ExtractData {
    type Output: DeserializeOwned;
    type Input: Serialize;
    fn get_file_path(&self) -> ManagerCoreResult<PathBuf>;
    fn get_file(&self) -> ManagerCoreResult<File> {
        Ok(File::open(self.get_file_path()?)?)
    }
    fn get_file_create(&self) -> ManagerCoreResult<File> {
        Ok(File::create(self.get_file_path()?)?)
    }
    fn get_buf_reader(&self) -> ManagerCoreResult<BufReader<File>> {
        Ok(BufReader::new(self.get_file()?))
    }
    fn get_buf_writer(&self) -> ManagerCoreResult<BufWriter<File>> {
        Ok(BufWriter::new(self.get_file_create()?))
    }
    fn get_data(&self) -> ManagerCoreResult<Self::Output> {
        Ok(serde_json::from_reader(self.get_buf_reader()?)?)
    }
    fn update(&self, input: Self::Input) -> ManagerCoreResult<()>;
    fn delete(&self) -> ManagerCoreResult<()>;
    fn is_there(&self) -> bool {
        self.get_data().is_ok()
    }
}
