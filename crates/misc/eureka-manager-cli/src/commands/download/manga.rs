use std::{fs::File, io::BufReader, path::PathBuf, str::FromStr};

use clap::Args;
use uuid::Uuid;

#[derive(Debug, Args)]
pub struct MangaDownloadArgs {
    /// Manga ids
    #[arg(long = "id")]
    pub ids: Vec<Uuid>,
    /// number
    #[arg(long, default_value_t = 5)]
    pub barrier: u16,
    #[arg(long)]
    pub id_text_file: Vec<PathBuf>,
}

impl MangaDownloadArgs {
    pub fn get_ids(&self) -> Vec<Uuid> {
        let mut ids = self.ids.clone();
        self.id_text_file
            .iter()
            .map(|e| (e, File::open(e)))
            .flat_map(|(path, res)| match res {
                Ok(file) => Some(id_list_txt_reader::IdListTxtReader::new(BufReader::new(
                    file,
                ))),
                Err(err) => {
                    log::error!("Cannot open the {} file: {}", path.to_string_lossy(), err);
                    None
                }
            })
            .flat_map(|file| file.flat_map(|s| Uuid::from_str(&s)))
            .for_each(|id| {
                ids.push(id);
            });
        ids
    }
}
