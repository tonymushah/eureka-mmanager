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
