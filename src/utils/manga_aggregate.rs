use std::io::Result;

use mangadex_api_schema::{ApiObject, v5::ChapterAttributes};

pub fn chapter_vec_to_chapter_aggregate_vec(input : Vec<ApiObject<ChapterAttributes>>) -> Result<()> {
    Ok(())
}