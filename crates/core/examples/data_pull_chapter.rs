use std::str::FromStr;

use clap::Parser;
use eureka_mmanager_core::{
    data_pulls::{chapter::ChapterListDataPullFilterParams, IntoFiltered, IntoSorted, Pull},
    DirsOptions,
};
use itertools::Itertools;
use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::{
    ChapterSortOrder, ContentRating, Language, MangaDexDateTime, RelationshipType,
};
use serde::{de::IntoDeserializer, Deserialize};
use uuid::Uuid;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    data_dir: Option<String>,
    #[arg(long)]
    title: Option<String>,
    #[arg(long)]
    groups: Vec<Uuid>,
    #[arg(long)]
    uploaders: Vec<Uuid>,
    #[arg(long)]
    volumes: Vec<String>,
    #[arg(long)]
    manga_ids: Vec<Uuid>,
    /// Chapter number in the series or volume.
    #[arg(long)]
    chapters: Vec<String>,
    #[arg(long)]
    translated_languages: Vec<Language>,
    #[arg(long)]
    original_languages: Vec<Language>,
    #[arg(long)]
    excluded_original_languages: Vec<Language>,
    #[arg(long)]
    content_rating: Vec<ContentRating>,
    /// Groups to exclude from the results.
    #[arg(long)]
    excluded_groups: Vec<Uuid>,
    /// Uploaders to exclude from the results.
    #[arg(long)]
    excluded_uploaders: Vec<Uuid>,
    #[arg(long, value_parser = mangadex_datetime)]
    created_at_since: Option<MangaDexDateTime>,
    /// DateTime string with following format: `YYYY-MM-DDTHH:MM:SS`.
    #[arg(long, value_parser = mangadex_datetime)]
    updated_at_since: Option<MangaDexDateTime>,
    /// DateTime string with following format: `YYYY-MM-DDTHH:MM:SS`.
    #[arg(long, value_parser = mangadex_datetime)]
    publish_at_since: Option<MangaDexDateTime>,
}

impl From<Cli> for ChapterListDataPullFilterParams {
    fn from(value: Cli) -> Self {
        Self {
            title: value.title,
            groups: value.groups,
            uploaders: value.uploaders,
            volumes: value.volumes,
            manga_ids: value.manga_ids,
            chapters: value.chapters,
            translated_languages: value.translated_languages,
            original_languages: value.original_languages,
            excluded_original_languages: value.excluded_original_languages,
            content_rating: value.content_rating,
            excluded_groups: value.excluded_groups,
            excluded_uploaders: value.excluded_uploaders,
            created_at_since: value.created_at_since,
            updated_at_since: value.updated_at_since,
            publish_at_since: value.publish_at_since,
        }
    }
}

fn mangadex_datetime(string: &str) -> Result<MangaDexDateTime, serde::de::value::Error> {
    MangaDexDateTime::deserialize(string.into_deserializer())
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();
    // Making a `DirOptions` instance
    let options = DirsOptions::new_from_data_dir(args.data_dir.clone().unwrap_or("data".into()));
    // verify if the directories is present and create them if they not exits
    options.verify_and_init()?;
    // "Pull"ing a single chapter
    if let Ok(chapter) = Pull::<ChapterObject, _>::pull(
        &options,
        Uuid::from_str("958ef762-9370-4c9f-afce-8238c31213dd")?,
    ) {
        println!(
            "{} [{:?}, {:?}, {:?}] exists",
            chapter.id,
            chapter.attributes.chapter,
            chapter.attributes.volume,
            chapter.find_relationships(RelationshipType::ScanlationGroup)
        );
    }
    // "Pull"ing all existing chapter
    for chapter in options
        .pull_all_chapter()?
        .flat_map(|res| match res {
            Ok(e) => Some(e),
            Err(err) => {
                eprintln!("{err}");
                None
            }
        })
        /* you can also filter it */
        .to_filtered(Into::<ChapterListDataPullFilterParams>::into(args))
        .collect_vec()
        .to_sorted(ChapterSortOrder::Chapter(
            mangadex_api_types_rust::OrderDirection::Ascending,
        ))
    {
        println!(
            "got chapter {:?} - vol. {:?} from {:?}",
            chapter.attributes.chapter,
            chapter.attributes.volume,
            chapter
                .find_first_relationships(RelationshipType::Manga)
                .map(|rel| { rel.id })
        )
    }
    Ok(())
}
