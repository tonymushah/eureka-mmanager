pub mod chapter;
pub mod cover;
pub mod manga;

use clap::Subcommand;

use super::{AsyncRun, AsyncRunContext};

#[derive(Debug, Subcommand)]
pub enum DeleteSubcommands {
    /// Delete Manga subcommand
    Manga(manga::MangaDeleteArgs),
    /// Delete Cover subcommand
    Cover(cover::CoverDeleteArgs),
    /// Delete Chapter subcommand
    Chapter(chapter::ChapterDeleteArgs),
}

impl AsyncRun for DeleteSubcommands {
    async fn run(&self, ctx: AsyncRunContext) -> anyhow::Result<()> {
        match self {
            DeleteSubcommands::Manga(manga_delete_args) => manga_delete_args.run(ctx).await,
            DeleteSubcommands::Cover(cover_delete_args) => cover_delete_args.run(ctx).await,
            DeleteSubcommands::Chapter(chapter_delete_args) => chapter_delete_args.run(ctx).await,
        }
    }
}
