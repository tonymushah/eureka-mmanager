# eureka-mmanager-core

It's the core piece for `eureka-mmanager` and `emdx` crate.

Providing an ergonomic API to interact with the eureka-mmanager filesytem structure.

## NOTICE: This library is still in developpement and not yet documented. Use it at your own risk

## Concepts

The `DirsOption` struct is the base of this entire repo. And you can only do two type of action with it:

- **Push** which is pushing data to the actual directories.
- **Pull** which is getting or retrieving data to the actual directories.

## Examples

### Pulling chapter data

```rust  
use eureka_mmanager_core::{
    data_pulls::{chapter::ChapterListDataPullFilterParams, IntoFiltered, Pull},
    DirsOptions,
};
use mangadex_api_schema_rust::v5::ChapterObject;
use mangadex_api_types_rust::{Language, RelationshipType};
use uuid::Uuid;

fn main() -> anyhow::Result<()> {
    // Making a `DirOptions` instance
    let options = DirsOptions::new_from_data_dir("data");
    // verify if the directories is present and create them if they not exits
    options.verify_and_init()?;
    // "Pull"ing a single chapter
    if let Ok(chapter) = Pull::<ChapterObject, _>::pull(&options, Uuid::new_v4()) {
        println!("{} exists", chapter.id);
    }
    // "Pull"ing all existing chapter
    for chapter in options
        .pull_all_chapter()?
        .flatten()
        /* you can also filter it */
        .to_filtered(ChapterListDataPullFilterParams {
            translated_languages: vec![Language::English],
            ..Default::default()
        })
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
```
