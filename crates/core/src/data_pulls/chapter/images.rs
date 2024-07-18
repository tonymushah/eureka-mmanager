use std::{fs::read_dir, path::Path};

use uuid::Uuid;

use crate::{data_pulls::Pull, DirsOptions, ManagerCoreResult};

#[derive(Debug, Clone, Hash, Default)]
#[cfg_attr(feature = "actix", derive(actix::MessageResponse))]
pub struct ChapterImagesData {
    pub data: Vec<String>,
    pub data_saver: Vec<String>,
}

impl ChapterImagesData {
    pub fn is_empty(&self) -> bool {
        self.data.is_empty() && self.data_saver.is_empty()
    }
}

fn m_read_dir<P: AsRef<Path>>(path: P) -> Vec<String> {
    read_dir(path)
        .map(|dir| {
            dir.flatten()
                .filter(|f| {
                    f.path().is_file()
                        && f.path()
                            .extension()
                            .map(|ext| ext != "json")
                            .unwrap_or_default()
                })
                .filter_map(|e| e.file_name().to_str().map(String::from))
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

fn string_f_usize_sort(data: &mut [String]) -> Result<(), regex::Error> {
    let regex = regex::Regex::new(r"\d+")?;
    data.sort_by(|a, b| {
        let a = regex
            .captures(a)
            .and_then(|c| c.get(0)?.as_str().parse::<usize>().ok());
        let b = regex
            .captures(b)
            .and_then(|c| c.get(0)?.as_str().parse::<usize>().ok());
        a.cmp(&b)
    });
    Ok(())
}

impl Pull<ChapterImagesData, Uuid> for DirsOptions {
    type Error = crate::Error;
    fn pull(&self, id: Uuid) -> ManagerCoreResult<ChapterImagesData> {
        let mut data = m_read_dir(self.chapters_id_data_add(id));
        string_f_usize_sort(&mut data)?;
        let mut data_saver = m_read_dir(self.chapters_id_data_saver_add(id));
        string_f_usize_sort(&mut data_saver)?;
        Ok(ChapterImagesData { data, data_saver })
    }
}
