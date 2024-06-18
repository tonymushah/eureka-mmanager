# mangadex-desktop-api2

[![Rust][rust-action-badge]][rust-action]

`mangadex-desktop-api2` is a library for downloading and managing titles, covers, and chapters from [MangaDex][mangadex].

This is built on top of the [`mangadex-api`][mangadex-api].
But unlike the SDK, it allows you to download, read, update and delete chapter images, covers and titles metadata from your local device.

It might also get a package management system, sees [#174][pms]

If you're building a MangaDex desktop app (like [Special Eureka][special-eureka]) or a private downloading service,
this library provide a bunch of features that might help you:

- Configurable download directory: With the [`DirsOptions`][dirs-options-api], you can change the
- It's asynchronous: This library is built on top of [actix] actors (not [actix-web])
  but it requires you to use [actix] system handler as an asynchronous runtime.

  Here is an example of using [actix] with [tauri]:
  
  ```rust
    #[actix::main]
    async fn main() {
        // Since [`actix`] is built on top of [`tokio`], we can use [`tokio::runtime::Handle::current()`]
        // to share the actix runtime with Tauri
        tauri::async_runtime::set(tokio::runtime::Handle::current());

        // bootstrap the tauri app...
        // tauri::Builder::default().run().unwrap();
    }
  ```

## Feature flags

None for now!

## Some concepts that you need to know

This library now uses [Actor model][actor-model] to track the download process in real-time.

Each downloading task: chapter / cover images downloading and title metadata is an actor and which I call `Task`.
You can listen to a task to know it's state like if it's pending, loading, success, or error.
Each task have an unique id, corresponding to what it's downloading.
Downloading a manga will spawn a [`MangaDownloadTask`][manga-download-task] actor, cover will spawn a [`CoverDownloadTask`][cover-download-task], etc...

A Task is managed by a manager corresponding by the task type,
which means that there is a [`ChapterDownloadManager`][chapter-download-mananger], a [`CoverDownloadManager`][cover-download-manager] and a [`MangaDownloadManager`][manga-download-manager].

To manage all of this there is a top level [`DownloadManger`][download-manager] that allows you to interact with the manager.
But it also contain an [inner state][inner-state] allows to interact with the underlying [MangaDex API Client][mangadex-api-client], [`DirOptions` API][dirs-options-api] and the [download history actor][history-service-api].

### `DirOptions` API

The [`DirOptions` API][dirs-options-api] manages every interaction to the filesystem.
You can get, create/update data and delete data like metadatas and images.

#### Getting data

To get data, it is done with the [`DataPulls`][data-pulls].

```rust
    use actix::prelude::*;
    use mangadex_api_types_rust::{MangaSortOrder, OrderDirection};
    /// Yes! we have a prelude module too.
    use mangadex_desktop_api2::prelude::*;

    fn main() -> anyhow::Result<()> {
        /// start a actix system
        let run = System::new();
        /// Runs your async code with `.block_on`
        run.block_on(async {
            /// Init the dir option api
            let options = DirsOptions::new_from_data_dir("data");
            /// Verify and init the required directories.
            /// This is mostly not required because `.start()` automatically call `.verify_and_init()`
            options.verify_and_init()?;
            /// init the actor
            let options_actor = options.start();
            /// init a data pull
            let data_pull = options_actor
                .get_manga_list()
                .await?
                /// Yes, you can sort data now
                .to_sorted(MangaSortOrder::Year(OrderDirection::Ascending))
                .await;
            /// Iterate over the results
            for manga in data_pull {
                println!("{:#?} - {has_failed}", manga.id);
                if let Some(year) = manga.attributes.year {
                    println!("year {year}",)
                }
            }
            Ok::<(), anyhow::Error>(())
        })?;
        Ok(())
    }
```

#### Create/update data (aka push)

To push data, it is done with the [`Push` trait][push-trait].

```rust
    use std::collections::HashMap;

    /// This example will illustrate how to push data to a
    /// You need to enable the `macros` feature for `actix` to make this example work.
    use actix::prelude::*;
    use mangadex_api_schema_rust::{
        v5::{
            AuthorAttributes, CoverAttributes, MangaAttributes, RelatedAttributes, Relationship,
            TagAttributes,
        },
        ApiObject,
    };
    use mangadex_api_types_rust::{
        ContentRating, Demographic, Language, MangaState, MangaStatus, RelationshipType, Tag,
    };
    use mangadex_desktop_api2::prelude::*;
    use url::Url;
    use uuid::Uuid;

    #[actix::main]
    async fn main() -> anyhow::Result<()> {
        // Init the dir options api
        let options = DirsOptions::new_from_data_dir("output").start();
        // Cover, author and artists is required as relationship
        let author = Relationship {
            id: Uuid::new_v4(),
            type_: RelationshipType::Author,
            related: None,
            attributes: Some(RelatedAttributes::Author(AuthorAttributes {
                name: String::from("Tony Mushah"),
                image_url: Some(String::from(
                    "https://avatars.githubusercontent.com/u/95529016?v=4",
                )),
                biography: Default::default(),
                twitter: Url::parse("https://twitter.com/tony_mushah").ok(),
                pixiv: None,
                melon_book: None,
                fan_box: None,
                booth: None,
                nico_video: None,
                skeb: None,
                fantia: None,
                tumblr: None,
                youtube: None,
                weibo: None,
                naver: None,
                namicomi: None,
                website: Url::parse("https://github.com/tonymushah").ok(),
                version: 1,
                created_at: Default::default(),
                updated_at: Default::default(),
            })),
        };
        let artist = {
            let mut author_clone = author.clone();
            author_clone.type_ = RelationshipType::Artist;
            author_clone
        };
        let cover = Relationship {
            id: Uuid::new_v4(),
            type_: RelationshipType::CoverArt,
            related: None,
            attributes: Some(RelatedAttributes::CoverArt(CoverAttributes {
                description: String::default(),
                locale: Some(Language::Japanese),
                volume: Some(String::from("1")),
                file_name: String::from("somecover.png"),
                created_at: Default::default(),
                updated_at: Default::default(),
                version: 1,
            })),
        };
        let my_manga = ApiObject {
            id: Uuid::new_v4(),
            type_: RelationshipType::Manga,
            attributes: MangaAttributes {
                // Totally an idea that i found myself :D
                title: HashMap::from([(Language::English, String::from("Dating a V-Tuber"))]),
                // Sorry, i use google traduction for this one.
                alt_titles: vec![HashMap::from([(Language::Japanese, String::from("VTuberとの出会い"))])],
                available_translated_languages: vec![Language::English, Language::French],
                // Hahaha... I wish it will got serialized very soon xD
                description: HashMap::from([(Language::English, String::from("For some reason, me #Some Guy# is dating \"Sakachi\", the biggest V-Tuber all over Japan. But we need to keep it a secret to not terminate her V-Tuber career. Follow your lovey-dovey story, it might be worth it to read it."))]),
                is_locked: false,
                links: None,
                original_language: Language::Malagasy,
                last_chapter: None,
                last_volume: None,
                publication_demographic: Some(Demographic::Shounen),
                state: MangaState::Published,
                status: MangaStatus::Ongoing,
                year: Some(2025),
                content_rating: Some(ContentRating::Suggestive),
                chapter_numbers_reset_on_new_volume: false,
                latest_uploaded_chapter: None,
                // You can put any tag that you want
                tags: vec![ApiObject {
                    id: Tag::Romance.into(),
                    type_: RelationshipType::Tag,
                    attributes: TagAttributes {
                        name: HashMap::from([(Language::English, Tag::Romance.to_string())]),
                        description: Default::default(),
                        group: Tag::Romance.into(),
                        version: 1
                    },
                    relationships: Default::default()
                }, ApiObject {
                    id: Tag::AwardWinning.into(),
                    type_: RelationshipType::Tag,
                    attributes: TagAttributes {
                        name: HashMap::from([(Language::English, Tag::AwardWinning.to_string())]),
                        description: Default::default(),
                        group: Tag::AwardWinning.into(),
                        version: 1
                    },
                    relationships: Default::default()
                }, ApiObject {
                    id: Tag::Drama.into(),
                    type_: RelationshipType::Tag,
                    attributes: TagAttributes {
                        name: HashMap::from([(Language::English, Tag::Drama.to_string())]),
                        description: Default::default(),
                        group: Tag::Drama.into(),
                        version: 1
                    },
                    relationships: Default::default()
                }, ApiObject {
                    id: Tag::SliceOfLife.into(),
                    type_: RelationshipType::Tag,
                    attributes: TagAttributes {
                        name: HashMap::from([(Language::English, Tag::SliceOfLife.to_string())]),
                        description: Default::default(),
                        group: Tag::SchoolLife.into(),
                        version: 1
                    },
                    relationships: Default::default()
                }],
                created_at: Default::default(),
                updated_at: Default::default(),
                version: 1
            },
            relationships: vec![author, artist, cover]
        };
        // Just call `.push()`
        options.push(my_manga).await?;
        Ok(())
    }

```

#### Deleting data

To delete data, it is done with the [`Delete` traits][delete-traits].

```rust
    use std::str::FromStr;

    use actix::prelude::*;
    use mangadex_desktop_api2::prelude::*;
    use tokio_stream::StreamExt;
    use uuid::Uuid;

    fn main() -> anyhow::Result<()> {
        // Init the actix system runner
        let run = System::new();
        run.block_on(async {
            // Start the option actor
            let options_actor = DirsOptions::new_from_data_dir("data").start();
            let manga_id = Uuid::from_str("b4c93297-b32f-4f90-b619-55456a38b0aa")?;
            // You can just call `.delete_manga(Uuid)` to delete a give manga
            let data = options_actor.delete_manga(manga_id).await?;
            // The `MangaDeleteData` consists of `covers` field which is the deleted covers ids
            // and `chapters` field which is the deleted chapters ids
            println!("{:#?}", data);
            // Get all the manga chapter
            let chapters: Vec<Uuid> = {
                let params = ChapterListDataPullFilterParams {
                    manga_id: Some(manga_id),
                    ..Default::default()
                };
                options_actor
                    .get_chapters()
                    .await?
                    .to_filtered(params)
                    .map(|o| o.id)
                    .collect()
                    .await
            };
            let covers: Vec<Uuid> = {
                let params = CoverListDataPullFilterParams {
                    manga_ids: [manga_id].into(),
                    ..Default::default()
                };
                options_actor
                    .get_covers()
                    .await?
                    .to_filtered(params)
                    .map(|o| o.id)
                    .collect()
                    .await
            };
            // check if there is no chapters left
            assert!(chapters.is_empty(), "Some chapter still remains");
            // check if there is no covers left
            assert!(covers.is_empty(), "Some covers still remains");
            Ok::<(), anyhow::Error>(())
        })?;
        Ok(())
    }
```

### The `DownloadHistory` API

Only purpose of the `DownloadHistory` API is to track download errors.
Which means that if you have an unfinished download or an failed download, you should be able to see it.
You can interact with it with the [`HistoryActorService`][history-service-api], but be careful when inserting or removing entries.
It could pontentialy break your app.

## Licence

Since v1, this package has now an MIT licence,
so it's your choice :).

[rust-action-badge]: https://github.com/tonymushah/eureka-mmanager/actions/workflows/rust.yml/badge.svg
[rust-action]: https://github.com/tonymushah/eureka-mmanager/actions/workflows/rust.yml
[mangadex]: https://mangadex.org
[mangadex-api]: https://github.com/tonymushah/mangadex-api
[pms]: https://github.com/tonymushah/eureka-mmanager/issues/174
[special-eureka]: https://github.com/tonymushah/special-eureka
[actix]: https://docs.rs/actix/latest/actix/
[actix-web]: https://docs.rs/actix-web/latest/actix_web/
[actor-model]: https://en.wikipedia.org/wiki/Actor_model
[download-manager]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download.rs
[manga-download-manager]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/manga.rs
[cover-download-manager]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/covers.rs
[chapter-download-mananger]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/chapter.rs
[cover-download-task]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/cover/task.rs
[manga-download-task]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/manga/task.rs
[inner-state]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/download/state.rs
[mangadex-api-client]: https://docs.rs/mangadex-api/latest/mangadex_api/v5/struct.MangaDexClient.html
[dirs-options-api]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/files_dirs.rs
[history-service-api]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/history/service.rs
[data-pulls]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/data_pulls.rs
[push-trait]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/data_push.rs
[delete-traits]: https://github.com/tonymushah/eureka-mmanager/blob/main/src/files_dirs/messages/delete.rs
[tauri]: https://tauri.app
