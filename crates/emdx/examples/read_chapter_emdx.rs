use std::fs::File;

use emdx::Archive;

fn main() -> anyhow::Result<()> {
    let mut archive = File::open("your_package.emdx")?;
    let mut emdx_package = Archive::from_reader(&mut archive)?;
    for chapter in emdx_package.chapter_pull(true)?.flatten() {
        println!("has chapter {}", chapter.id);
    }
    Ok(())
}
